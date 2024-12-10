use arrayvec::ArrayString;
const TITLE_LENGTH: usize = 60;

#[derive(Debug)]
struct Title {
    value: ArrayString<MAX_LENGTH>,
}

impl Title {
    fn new(value: &str) -> Result<Self, &'static str> {
        if value.len() > MAX_LENGTH {
            return Err("String exceeds maximum length");
        }
        let array_string = ArrayString::<MAX_LENGTH>::from(value).map_err(|_| "Invalid UTF-8")?;
        Ok(Self {
            value: array_string,
        })
    }

    fn value(&self) -> &str {
        &self.value
    }
}

fn main() {
    let valid_string = LimitedString::new("Hello");
    match valid_string {
        Ok(s) => println!("Valid string: {}", s.value()),
        Err(e) => println!("Error: {}", e),
    }

    let invalid_string = LimitedString::new("This is a very long string");
    match invalid_string {
        Ok(s) => println!("Valid string: {}", s.value()),
        Err(e) => println!("Error: {}", e),
    }
}
