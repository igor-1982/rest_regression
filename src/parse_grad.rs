/*!
    Parse gradient output

    Sample input:
    ```none
    ------ Output gradient [a.u.] ------
    N       -0.00401894     -0.00063106      0.01134897
    H        0.02006915     -0.00015773      0.00282861
    H       -0.00802486     -0.01678341     -0.00804460
    H       -0.00802536      0.01757220     -0.00613298

    ------------------------------------
    ```

    Rules:
    - Wrapped by `-- Output gradient [***] --` and long dashes
    - Gradient values are the last 3 columns

    This file handles the extraction of the last three columns.
*/

/// Parse the gradient output from a string
///
/// Sample input:
///
/// ```none
/// N       -0.00401894     -0.00063106      0.01134897
/// H        0.02006915     -0.00015773      0.00282861
/// H       -0.00802486     -0.01678341     -0.00804460
/// H       -0.00802536      0.01757220     -0.00613298
/// ```
///
/// Sample output:
///
/// ```none
/// vec![-0.00401894, -0.00063106, 0.01134897, 0.02006915, -0.00015773, ...]
/// ```
pub fn parse_grad(token: &String) -> Vec<f64> {
    let mut grad = Vec::new();
    let mut lines = token.lines();

    // Skip the first line
    lines.next();

    // Read the rest of the lines
    for line in lines {
        // Split the line into tokens
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < 3 {
            continue; // Skip lines that don't have enough tokens
        }
        // Parse the last three tokens as f64 and add them to the vector
        if let (Ok(x), Ok(y), Ok(z)) = (
            tokens[tokens.len() - 3].parse::<f64>(),
            tokens[tokens.len() - 2].parse::<f64>(),
            tokens[tokens.len() - 1].parse::<f64>(),
        ) {
            grad.push(x);
            grad.push(y);
            grad.push(z);
        }
    }
    grad
}
