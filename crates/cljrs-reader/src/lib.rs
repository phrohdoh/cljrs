pub mod value;
pub mod keyword;
pub mod char_reader;

use archery::SharedPointerKind;
use value::Value;
use char_reader::CharReader;

pub trait ReadClj<P: SharedPointerKind> {
    fn try_read_one(&mut self) -> ReadResult<P>;
    fn try_read_comment(&mut self) -> ReadResult<P>;
    fn try_read_symbol(&mut self) -> ReadResult<P>;
    fn try_read_keyword(&mut self) -> ReadResult<P>;
    fn try_read_string(&mut self) -> ReadResult<P>;
    fn try_read_list(&mut self) -> ReadResult<P>;
    fn try_read_vect(&mut self) -> ReadResult<P>;
    fn try_read_set(&mut self) -> ReadResult<P>;
    fn try_read_map(&mut self) -> ReadResult<P>;
}

pub fn reader<P: SharedPointerKind>(s: &str) -> Option<Box<dyn ReadClj<P> + '_>> {
    match CharReader::try_from_str(s) {
        Some(rdr) => Some(Box::new(rdr)),
        _ => None,
    }
}

pub type ReadInput<'input, T, Span> = WithSpan<Span, &'input T>;
pub type ReadResult<P> = Result<ReadOutput<P>, ReadError>;
pub type ReadOutput<P> = Option<SpanValue<P>>;

pub type SpanValue<P> = WithSpan<ByteIndexSpan, Value<P>>;

#[derive(Debug, Clone)]
pub struct WithSpan<S, D> {
    pub span: S,
    pub data: D,
}

impl<P: SharedPointerKind> SpanValue<P> {
    pub fn span(self) -> ByteIndexSpan {
        self.span
    }
    pub fn data(self) -> Value<P> {
        self.data
    }
}


pub type BeginByteIdx = usize;
pub type   EndByteIdx = usize; // inclusive, so use ..=
pub type ByteIndexSpan = (BeginByteIdx, EndByteIdx);

#[derive(Debug, Clone, PartialEq)]
pub enum ReadError {
    InsufficientInput(ByteIndexSpan),
    InvalidInput(ByteIndexSpan),
    UnclosedCollection(ByteIndexSpan)
}

impl ReadError {
    pub fn insufficient_input(span: ByteIndexSpan) -> Self {
        Self::InsufficientInput(span)
    }
    pub fn is_insufficient_input(&self) -> bool {
        match self {
            Self::InsufficientInput(..) => true,
            _ => false,
        }
    }
    pub fn invalid_input(span: ByteIndexSpan) -> Self {
        Self::InvalidInput(span)
    }
    pub fn is_invalid_input(&self) -> bool {
        match self {
            Self::InvalidInput(..) => true,
            _ => false,
        }
    }
    pub fn unclosed_collection(span: ByteIndexSpan) -> Self {
        Self::UnclosedCollection(span)
    }
    pub fn is_unclosed_collection(&self) -> bool {
        match self {
            Self::UnclosedCollection(_) => true,
            _ => false,
        }
    }
}