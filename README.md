# colain

[![Crates.io](https://img.shields.io/crates/v/colain.svg)](https://crates.io/crates/colain)
[![Documentation](https://docs.rs/colain/badge.svg)](https://docs.rs/colain/)

Parser for the Common Layer Interface (.cli) file [format.](http://web.archive.org/web/19970617041930/http://www.cranfield.ac.uk/aero/rapid/CLI/cli_v20.html)

**Note:** This library does not yet parse ASCII files. 

#### Requires `rustc` `1.51.0+`

### Example

```rust
use std::fs::File;
use std::io::prelude::*;
use colain::{
    CLI,
    clitype::{LongCLI, ShortCLI},
    Point // import the Point trait to provide access via .x() and .y()
};

// Load the file
let mut buf: Vec<u8> = Vec::new();
File::open("example.cli").unwrap().read_to_end(&mut buf).unwrap();

// Parse the file
let model = CLI::<LongCLI>::new(&buf).unwrap();

// for each layer
for layer in model.iter() {
     // for each loop in the layer
     for a_loop in layer.iter_loops() {
         // for each point in the loop
         for point in a_loop.iter() {
             let x = point.x();
             let y = point.y();
         }
     }
 }
```

### Licence

Licensed under Apache 2.0

### Contributing
Please feel free to submit a PR. 

Additionally, .cli files would be very useful for testing, please consider 
submitting any files to the `testfiles` directory as a PR. 

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.


#### Todo

- [ ] Switch to Iterator API
- [ ] Support remaining header commands
- [ ] Tests 
- [ ] PR for ASCII file support is welcome
