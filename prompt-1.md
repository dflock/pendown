I would like to make a version of the logo programming language, in rust. I would like this in two broad parts:

- the logo implementation crate & cli
- a GUI application, using GPUI & GPUI-kit, that provides a frontend playground for using logo

The frontend will need a big blank canvas for the logo turtle to draw onto. The initial question to answer is does gpui or gpui-kit provide this some kind of canvas component that would work for this?

I have put reference materials in the ./refs folder - this has the source code for gpui in ./refs/zed/crates/gpui, as well as gpui-kit in ./refs/gpui-kit
