# choice-string

Provides utilities to parse selections (indices and ranges) from a string

---

## Features

### Parsing selection strings
For this crate, the format of the selection string can be understood through the following example:
```
The following packages are available:
(1) good-package    (2) bad-package     (3) decent-package
(4) awful-package   (5) amazing-package (6) package6
(7) package7        (8) package8    
Select which packages to install:
>> 1 3 5 6-8

The following packages are being installed:
good-package, decent-package, amazing-package, package6,
package7, package8
...
```

In this case, the user has selected indices 1, 3, 5, and all between (inclusive) 6 through 8.

### Simple API

```rust
let selection =  choice_string::parse("1, 2, 3, 4-8").expect("parse error");
assert!(selection.contains_item(5));
```

### Flexible syntax

The delimiters are flexible. Any of the following work:
* ` ` - A space
* `,` - A comma
* `;` - A semicolon

You can also use any combination of the above in a single string, and even mix as single separators.

For example:
* `1, 2, 3,5 6-9`
* `1-8 11, 12`
* `1,2,3,4`
* `1;2;3;4`

### Condenses to minimal representation

After parsing, the provided ranges are condensed by a union operation.

For example:
* `1, 2, 3, 4-5, 11` -> `1-5, 11`