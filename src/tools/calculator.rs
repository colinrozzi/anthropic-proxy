use crate::tools::ToolDefinition;
use serde_json::json;

/// Returns a definition for a calculator tool that can perform basic arithmetic
pub fn get_calculator_tool() -> ToolDefinition {
    ToolDefinition {
        name: "calculate".to_string(),
        description: "Performs basic arithmetic calculations. This tool can add, subtract, multiply, and divide numbers. It supports parentheses for grouping operations. It does not support advanced functions, trigonometry, or variables. Use this tool when the user asks for mathematical calculations or when you need to compute a numerical result.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate. For example: '2 + 2', '(3 * 4) / 2', '15 - 6'"
                }
            },
            "required": ["expression"]
        }),
    }
}

/// Evaluates a simple arithmetic expression
pub fn evaluate_expression(expression: &str) -> Result<f64, String> {
    // Remove whitespace
    let expr = expression.replace(" ", "");
    
    // Parse and evaluate the expression
    simple_evaluate(&expr)
}

// Very simple expression evaluator for basic arithmetic
// This is a naive implementation. A real implementation would use a proper math expression parser
fn simple_evaluate(expression: &str) -> Result<f64, String> {
    // Handle parentheses first
    let mut expr = expression.to_string();
    while let Some(start) = expr.find('(') {
        let mut depth = 1;
        let mut end = start + 1;
        
        while depth > 0 && end < expr.len() {
            match expr.chars().nth(end) {
                Some('(') => depth += 1,
                Some(')') => depth -= 1,
                None => return Err("Mismatched parentheses".to_string()),
                _ => {}
            }
            if depth > 0 {
                end += 1;
            }
        }
        
        if depth > 0 {
            return Err("Mismatched parentheses".to_string());
        }
        
        let inner_result = simple_evaluate(&expr[start+1..end])?;
        let before = &expr[0..start];
        let after = if end + 1 < expr.len() { &expr[end+1..] } else { "" };
        expr = format!("{}{}{}", before, inner_result, after);
    }
    
    // Split by addition and subtraction
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut sign = 1.0;
    
    // Handle potential leading +/- sign
    let mut chars = expr.chars().peekable();
    if let Some(&c) = chars.peek() {
        if c == '-' {
            sign = -1.0;
            chars.next();
        } else if c == '+' {
            chars.next();
        }
    }
    
    for c in chars {
        match c {
            '+' => {
                if !current.is_empty() {
                    parts.push(sign * parse_term(&current)?);
                    current.clear();
                }
                sign = 1.0;
            },
            '-' => {
                if !current.is_empty() {
                    parts.push(sign * parse_term(&current)?);
                    current.clear();
                }
                sign = -1.0;
            },
            _ => current.push(c),
        }
    }
    
    if !current.is_empty() {
        parts.push(sign * parse_term(&current)?);
    }
    
    // Sum all parts
    Ok(parts.iter().sum())
}

fn parse_term(term: &str) -> Result<f64, String> {
    // Split by multiplication and division
    let mut result = 1.0;
    let mut current = String::new();
    let mut operation = '*';
    
    for c in term.chars() {
        match c {
            '*' | '/' => {
                if !current.is_empty() {
                    let value = parse_factor(&current)?;
                    match operation {
                        '*' => result *= value,
                        '/' => {
                            if value == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            result /= value;
                        },
                        _ => unreachable!(),
                    }
                    current.clear();
                }
                operation = c;
            },
            _ => current.push(c),
        }
    }
    
    if !current.is_empty() {
        let value = parse_factor(&current)?;
        match operation {
            '*' => result *= value,
            '/' => {
                if value == 0.0 {
                    return Err("Division by zero".to_string());
                }
                result /= value;
            },
            _ => unreachable!(),
        }
    }
    
    Ok(result)
}

fn parse_factor(factor: &str) -> Result<f64, String> {
    // Try to parse as a number
    match factor.parse::<f64>() {
        Ok(num) => Ok(num),
        Err(_) => Err(format!("Invalid number or expression: {}", factor)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_addition() {
        assert_eq!(evaluate_expression("2 + 3").unwrap(), 5.0);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(evaluate_expression("10 - 4").unwrap(), 6.0);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(evaluate_expression("5 * 3").unwrap(), 15.0);
    }

    #[test]
    fn test_division() {
        assert_eq!(evaluate_expression("20 / 4").unwrap(), 5.0);
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(evaluate_expression("2 + 3 * (4 - 1) / 3").unwrap(), 5.0);
    }

    #[test]
    fn test_division_by_zero() {
        assert!(evaluate_expression("5 / 0").is_err());
    }

    #[test]
    fn test_invalid_expression() {
        assert!(evaluate_expression("5 + abc").is_err());
    }
}
