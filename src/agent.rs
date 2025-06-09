// TODO

type Int = i32;

/// A simple function that adds two integers.
pub fn some_agent_fn(x: Int, y: Int) -> Int {
    x + y
}

#[cfg(test)]
mod tests {
    use super::*; // Import the functions from the parent module

    #[test]
    fn test_some_agent_fn() {
        assert_eq!(some_agent_fn(2, 3), 5);
        assert_eq!(some_agent_fn(-1, 1), 0);
        assert_eq!(some_agent_fn(0, 0), 0);
    }
}
