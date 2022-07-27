# ryml: Rapid YAML Bindings for Rust

![GPL 3](https://img.shields.io/crates/l/ryml)
![Crates.io version](https://img.shields.io/crates/v/ryml)
![GitHub issues](https://img.shields.io/github/issues/NiceneNerd/ryml)

[Rapid YAML/ryml](https://github.com/biojppm/rapidyaml) is a C++ library to
parse and emit YAML, and do it fast. The `ryml` crate provides fairly thin
but safe (I think) FFI bindings to bring this power to Rust.

This crate is currently is early, early alpha. Not all of the C++ API is
covered, and some of it probably will not be, but the core functionality is
all in place (indeed, much more than the official Python bindings).

## Usage

A basic example of how to use this crate:

```rust
use ryml::Tree;

static SRC: &str = r#"
Hello: World
Names: [Caleb, Voronwe, 'Nicene Nerd']
Credo: !!apostolicus
 - in Deum, Patrem omnipotentem
 - et in Iesum Christum, Filium eius unicum
 - in Spiritum Sanctum
"#;

let mut tree = Tree::parse(SRC)?;
assert_eq!(10, tree.len());

// First, the low-level, index-based API
let root_id = tree.root_id()?;
assert_eq!(root_id, 0);
assert_eq!(tree.num_children(root_id)?, 3);
for i in 0..tree.num_children(root_id)? {
  let child_id = tree.child_at(root_id, i)?;
  println!("{}", tree.key_scalar(child_id)?.scalar); // "Hello", "Names", "Credo"
}

// Next, the high-level, NodeRef API
let world = tree.root_ref()?.get("Hello")?;
assert_eq!(world.val()?, "World");
let credo = tree.root_ref()?.get("Credo")?;
assert!(credo.is_seq()?);
assert_eq!(credo.val_tag()?, "!!apostolicus");    
for node in credo.iter()? {
  println!("{}", node.val()?);
}

// Mutate the tree
{
   let mut root_ref_mut = tree.root_ref_mut()?;
   let mut credo = root_ref_mut.get_mut("Credo")?;
   let mut amen = credo.get_mut(3)?; // A new index
   amen.set_val("Amen")?;
}

// Serialize the tree back to a string
let new_text = tree.emit()?;

static END: &str = r#"Hello: World
Names:
 - Caleb
 - Voronwe
 - 'Nicene Nerd'
Credo: !!apostolicus
 - 'in Deum, Patrem omnipotentem'
 - 'et in Iesum Christum, Filium eius unicum'
 - in Spiritum Sanctum
 - Amen
"#;

assert_eq!(new_text, END);
```

For more usage information, see the [full documentation](https://docs.rs/ryml).

## Contributing

Issue tracker: https://github.com/NiceneNerd/ryml/issues  
Source code: https://github.com/NiceneNerd/ryml

Some welcome to-dos:
- Anything that would improve error handling
- `no_std` support is an eventual goal
- Safety improvements
- Could definitely use more tests and such

## License

This project is licensed under the [GPLv3+ license](https://www.gnu.org/licenses/gpl-3.0.en.html).
The original ryml library is licensed under the [MIT license](https://github.com/biojppm/rapidyaml/blob/master/LICENSE.txt).
