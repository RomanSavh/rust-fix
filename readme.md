# Rust-fix

A simple implementation of FIX builder for Rust.

- Auto calculate of body len and check sum
- Checksum validation
- TagsOrder save

## Example

A basic cases.

```rust,no_run
use rust_fix::FixMessageBuilder;

fn main() {
    let mut new_message = FixMessageBuilder::new("FIX.4.4", "A");
    new_message.with_value(34, "1");
    new_message.with_value(49, "test");
    new_message.with_value(55, "test");

     //formated_message = 8=FIX.4.4|9=84|35=A|34=1|49=test|55=test|10=039|
    let formated_message = new_message.to_string();
    let bytes = new_message.as_bytes();

    let serialize_message = FixMessageBuilder::from_bytes(&bytes, true);
}
```

## License

Rust fix is provided under the MIT license. See [LICENSE](LICENSE).