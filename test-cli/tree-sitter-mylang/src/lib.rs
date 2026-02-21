use adze::Grammar;

#[adze::grammar("mylang")]
pub struct mylang;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let _ = mylang::LANGUAGE;
    }
}
