# rust-git
Version control system written in rust.

## Usage
1. Run config and select folders to ignore: `cargo run config -i='target .git'`. 
2. Init `cargo run init`.
3. To make commit type: `cargo run commit -t='my simple commit title'`. -t means title alternatively use --title=
4. To see logs type `cargo run log`
5. To see the status of changes type `cargo run status`
6. If you want to go back to some commit just type: `cargo run back -t=dc093dc4-af1e-411a-b41a-42866869615a` In -t you type id of commit which you want to go back. You get that id from log.
