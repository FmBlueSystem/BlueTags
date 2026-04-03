pub fn to_decade(year: u32) -> String {
    format!("{}0s", year / 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decade() {
        assert_eq!(to_decade(1997), "1990s");
        assert_eq!(to_decade(2003), "2000s");
        assert_eq!(to_decade(1980), "1980s");
    }
}
