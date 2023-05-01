use std::fmt;

pub struct Dbg<T>(pub T);

trait FormatProxy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<T> FormatProxy for Dbg<T> {
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<unknown value>")
    }
}

impl<T> FormatProxy for Dbg<T>
where
    T: fmt::Debug,
{
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> fmt::Debug for Dbg<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        FormatProxy::fmt(self, f)
    }
}
