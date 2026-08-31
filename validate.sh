#!/usr/bin/env bash
# =============================================================================
# REST 标准工程验证入口 (Standard Engineering Validation Entrypoint)
#
# 规则（必须遵守）：
#   1. 运行前先阅读本目录下的 MPI_TEST_SCOPE.md —— 它是 MPI 测试范围声明。
#   2. rayon-only 模式 (-n 1)：可跑 bench_pool 全量任务。
#   3. MPI 模式 (--mpi)：默认只跑声明范围内目录 (scf, gradient)；
#      若显式指定的测试超出声明范围，本脚本会打印 WARNING。
#   4. 开发标准：任何功能开发完成后，必须运行本脚本（rayon-only 全量 +
#      MPI 范围内全量）全部通过，才能视为功能开发成功。
#
# 用法:
#   ./validate.sh                        # rayon-only 全量 (debug 构建)
#   ./validate.sh --mpi                  # MPI 模式，自动限定在声明范围内
#   ./validate.sh --mpi -n 4             # MPI 4 进程
#   ./validate.sh --mpi -f "NH3_SCAN"    # MPI 定向测试（超范围给 warning）
#   ./validate.sh --scalapack            # MPI + 强制 ScaLAPACK 分布式路径（注入关键词）
#   ./validate.sh -f "NH3_SCAN,NH3_SVWN" # rayon-only 定向测试
#   ./validate.sh -c release             # 使用 release 构建
#   ./validate.sh -p /path/to/rest       # 指定 rest 二进制
#   ./validate.sh --dry-run              # 只列出将运行的任务
# =============================================================================
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCOPE_FILE="$SCRIPT_DIR/MPI_TEST_SCOPE.md"
BENCH_ROOT="$SCRIPT_DIR/bench_pool"

# ---------------- 默认值 ----------------
MODE="debug"            # debug | release
N_MPI=1                 # 1 = rayon-only
FILTER=""               # 空 = 全部
TIMEOUT=1800
REST_BIN=""
DRY_RUN=0
SCALAPACK=0
EXTRA_ARGS=()

# ---------------- 参数解析 ----------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --mpi) N_MPI=2 ;;
        --scalapack) SCALAPACK=1; N_MPI=2 ;;
        -n) N_MPI="$2"; shift ;;
        -c|--mode) MODE="$2"; shift ;;
        -p|--rest_path) REST_BIN="$2"; shift ;;
        -f|--filter) FILTER="$2"; shift ;;
        --timeout) TIMEOUT="$2"; shift ;;
        --dry-run) DRY_RUN=1 ;;
        -h|--help)
            sed -n '2,32p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) EXTRA_ARGS+=("$1") ;;
    esac
    shift
done

# ---------------- 环境准备 ----------------
export REST_BASIS_DIR="${REST_BASIS_DIR:-$SCRIPT_DIR/../rest/basis-set-pool}"
if [[ ! -d "$REST_BASIS_DIR" ]]; then
    echo "WARNING: REST_BASIS_DIR=$REST_BASIS_DIR does not exist; HI-series tasks may fail via BSE download."
fi
# REST 约定：所有相关库（hdf5、scalapack、openblas 等）统一安装/链接在 $REST_EXT_DIR。
# 运行时只需将 $REST_EXT_DIR 置于 LD_LIBRARY_PATH 最前（hdf5 为含 H5Pset_fapl_direct 的
# 310.5.1，libblas/liblapack 统一指向 OpenBLAS，scalapack 依赖链已齐）。
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}"
if [[ -d "$SCRIPT_DIR/../lib" && "$LD_LIBRARY_PATH" != *"$SCRIPT_DIR/../lib"* ]]; then
    if [[ -z "$LD_LIBRARY_PATH" ]]; then
        export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib"
    else
        export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"
    fi
fi

# ---------------- rest 二进制 ----------------
if [[ -z "$REST_BIN" ]]; then
    REST_BIN="$SCRIPT_DIR/../target/$MODE/rest"
fi
if [[ ! -x "$REST_BIN" ]]; then
    echo "ERROR: REST binary not found: $REST_BIN (build first or pass -p)" >&2
    exit 2
fi

# ---------------- 范围检查（MPI 模式） ----------------
if [[ "$N_MPI" -ge 2 ]]; then
    if [[ ! -f "$SCOPE_FILE" ]]; then
        echo "WARNING: $SCOPE_FILE not found; cannot enforce MPI test scope." >&2
    else
        # 从声明文件中提取范围内的目录（表格第一列, bench_pool/xxx 形式）
        SCOPE_DIRS=()
        while IFS= read -r line; do
            line="$(echo "$line" | sed 's|bench_pool/||; s|/$||')"
            [[ -n "$line" ]] && SCOPE_DIRS+=("$line")
        done < <(sed -n '/当前 MPI 测试范围/,/范围外目录/p' "$SCOPE_FILE" \
                 | grep -oE 'bench_pool/[a-z_]+/')
        if [[ ${#SCOPE_DIRS[@]} -eq 0 ]]; then
            echo "WARNING: could not parse scope dirs from $SCOPE_FILE" >&2
        fi
    fi

    if [[ -n "$FILTER" ]]; then
        # 逐项检查 filter 中的任务是否位于声明范围内
        IFS=',' read -ra items <<< "$FILTER"
        for item in "${items[@]}"; do
            item="$(echo "$item" | xargs)"   # trim
            found=""
            while IFS= read -r d; do
                rel="${d#$BENCH_ROOT/}"
                if [[ "${d##*/}" == "$item" ]]; then
                    found="$rel"
                    break
                fi
            done < <(find "$BENCH_ROOT" -maxdepth 2 -type d -name "$item" 2>/dev/null)
            if [[ -n "$found" ]]; then
                in_scope=0
                for s in "${SCOPE_DIRS[@]:-}"; do
                    if [[ "$found" == "$s"* ]]; then in_scope=1; break; fi
                done
                if [[ "$in_scope" -eq 0 ]]; then
                    echo "WARNING: MPI test '$item' ($found) is OUTSIDE the declared scope in $SCOPE_FILE ($(IFS=,; echo "${SCOPE_DIRS[*]}"))."
                fi
            else
                echo "WARNING: MPI filter item '$item' matches no bench_pool task dir; it may be a tag selecting tasks outside the declared scope."
            fi
        done
    else
        # 未指定 filter：MPI 模式自动限定在声明范围内
        FILTER="$(IFS=,; echo "${SCOPE_DIRS[*]}" | sed 's|bench_pool/||g; s|/$||g')"
        echo "MPI mode: scope-limited to: $FILTER (per $SCOPE_FILE)"
    fi
fi

# ---------------- 运行 ----------------
cd "$SCRIPT_DIR" || exit 2
if [[ "$DRY_RUN" -eq 0 ]]; then
    # 清空旧验证产物，避免范围外/旧任务 log 混淆本次结果
    rm -rf work_pool
    mkdir -p work_pool
fi
ARGS=(
    --regression_directory "$BENCH_ROOT"
    --working_directory work_pool
    --rest_path "$REST_BIN"
    --n_mpi "$N_MPI"
    --timeout "$TIMEOUT"
    --durations 10
)
if [[ -n "$FILTER" ]]; then
    ARGS+=(--filter "$FILTER")
fi
if [[ "$SCALAPACK" -eq 1 ]]; then
    ARGS+=(--scalapack)
fi
if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
    ARGS+=("${EXTRA_ARGS[@]}")
fi

echo "============================================================"
echo "REST validation | binary: $REST_BIN | n_mpi: $N_MPI | scalapack: $SCALAPACK | filter: ${FILTER:-ALL}"
echo "============================================================"

if [[ "$DRY_RUN" -eq 1 ]]; then
    cargo run --quiet -- "${ARGS[@]}" --dry-run
else
    cargo run --quiet -- "${ARGS[@]}"
fi
rc=$?
if [[ $rc -eq 0 ]]; then
    echo "VALIDATION PASSED (exit 0)"
else
    echo "VALIDATION FAILED (exit $rc)"
fi
exit $rc
