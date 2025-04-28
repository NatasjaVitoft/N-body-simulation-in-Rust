

#[cfg(test)]
mod tests {
    use crate::mass_to_hue;

    #[test]
    fn test_hue_conversion_1() {
        assert_eq!(mass_to_hue(1.0, 1.0, 1.0), 1.0);
    }
    #[test]
    fn test_hue_conversion_5000() {
        assert_eq!(mass_to_hue(5000.0, 5000.0, 5000.0), 1.0);
    }
    #[test]
    fn test_hue_conversion_10() {
        assert_eq!(mass_to_hue(2500.0, 0.0, 5000.0), 0.5);
    }
    
}