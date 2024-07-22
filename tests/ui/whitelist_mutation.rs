#![warn(clippy::fields_mutated_by_whitelist)]

pub struct TestStruct {
    #[mutatedby("allowed_function")]
    field: u8,
}

impl TestStruct {
    fn allowed_function(&mut self) {
        self.field = 1;
    }

    fn disallowed_function(&mut self) {
        self.field += 2; // Should trigger a lint warning
        panic!("Hi!");
    }
}

fn main() {
    let mut ts = TestStruct { field: 0 };
    ts.allowed_function();
    ts.disallowed_function(); // This should trigger a lint warning
}

