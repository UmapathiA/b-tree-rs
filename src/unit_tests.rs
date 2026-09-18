#[cfg(test)]
mod tests {
    use crate::add;

    #[test]
    fn test_addition() {
        println!("Some tests");
        assert_eq!(add(2, 3), 5);
    }
}
