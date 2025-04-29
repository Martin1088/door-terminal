use vte::{Parser, Perform};

pub fn strip_ansi_codes(input: &str) -> String {
    struct CleanText {
        output: String,
    }

    impl Perform for CleanText {
        fn print(&mut self, c: char) {
            self.output.push(c);
        }

        // Ignore all ANSI control sequences
        fn execute(&mut self, byte: u8) {
            match byte {
                b'\n' | b'\r' | b'\t' => {
                    self.output.push(byte as char);
                }
                _ => {}
            }
        }
        fn csi_dispatch(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
        fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
        fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
        fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
        fn put(&mut self, _byte: u8) {}
        fn unhook(&mut self) {}
    }

    let mut parser = Parser::new();
    let mut performer = CleanText {
        output: String::new(),
    };
    
    parser.advance(&mut performer, input.as_bytes());
    let text = performer.output.trim_end().to_string();
    normalize_spacing(&text)
}

fn normalize_spacing(text: &str) -> String {
    let re = regex::Regex::new(r"[ ]{2,}").unwrap();
    re.replace_all(text, "\t").to_string()
}