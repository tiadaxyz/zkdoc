# `zkdoc_sdk`

## Usage

First, add it with cargo like so:

```bash
cargo add zkdoc_sdk
```

Now, you can start using it!

```rust
use zkdoc_sdk::services::services::{
    generate_proof, get_file_commitment_and_selected_row, get_selected_row, verify_correct_selector,
};

fn main() {
  // Generate a commitment, and save it somewhere for verification later
  let commitment = get_file_commitment_and_selected_row(
    row_titles.to_owned(),
    row_contents.to_owned(),
    row_selectors.to_owned(),
  );

  // Generate proofs here
  let proof = generate_proof(
      row_titles.to_owned(),
      row_contents.to_owned(),
      row_selectors.to_owned(),
  );

  // Verify proofs like so
  let row_accumulator = get_selected_row(row_title.to_owned(), row_content.to_owned());
  let is_valid = verify_correct_selector(
      commitment,
      row_accumulator,
      proof,
  );

}
```

## API documentation

The full api docs is available at [doc.rs](https://docs.rs/zkdoc_sdk/0.0.0/zkdoc_sdk).