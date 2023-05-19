use crate::{
        keyword::Keyword,
        value::{Value, ValuePtr},
        ByteIndexSpan, ReadClj, ReadError, ReadResult, SpanValue, WithSpan,
};
use cljrs_core::symbol::Symbol;
use archery::SharedPointerKind;

type ByteIdx = usize;
type SrcIdx = usize;

#[derive(Debug)]
pub struct CharReader<'i> {
    _src: &'i str,
    src: Vec<(ByteIdx, char)>,
    src_idx: SrcIdx,
}

impl<'i> CharReader<'i> {
    pub fn try_from_str(s: &'i str) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Self {
                src: s.char_indices().collect(),
                _src: s,
                src_idx: 0,
            }
            .into()
        }
    }

    pub fn is_at_end(&mut self) -> bool {
        self.src_idx == self.src.len() - 1
    }

    pub fn is_beyond_end(&mut self) -> bool {
        self.src_idx > self.src.len() - 1
    }

    pub fn at(&mut self, src_idx: SrcIdx) -> Option<(ByteIdx, char)> {
        self.src.get(src_idx).copied()
    }

    pub fn byte_idx_at(&mut self, src_idx: SrcIdx) -> Option<ByteIdx> {
        self.at(src_idx).map(|(byte_idx, _)| byte_idx)
    }

    pub fn current(&mut self) -> Option<(ByteIdx, char)> {
        self.at(self.src_idx)
    }

    pub fn current_byte_idx(&mut self) -> Option<ByteIdx> {
        self.current().map(|(byte_idx, _)| byte_idx)
    }

    pub fn current_char(&mut self) -> Option<char> {
        self.current().map(|(_, ch)| ch)
    }

    pub fn current_char_eq(&mut self, eq: char) -> bool {
        self.current_char().map(|ch| ch == eq).unwrap_or(false)
    }

    pub fn advance(&mut self) {
        if !self.is_beyond_end() {
            self.src_idx += 1;
        }
    }

    pub fn peek(&mut self) -> Option<(ByteIdx, char)> {
        self.at(self.src_idx + 1)
    }

    pub fn peek_byte_idx(&mut self) -> Option<ByteIdx> {
        self.peek().map(|(byte_idx, _)| byte_idx)
    }

    pub fn peek_char(&mut self) -> Option<char> {
        self.at(self.src_idx + 1).map(|(_byte_idx, ch)| ch)
    }

    pub fn peek_char_eq(&mut self, eq: char) -> bool {
        self.peek_char().map(|ch| ch == eq).unwrap_or(false)
    }

    pub fn skip_whitespaces(&mut self) {
        while let Some((_, ch)) = self.current() {
            if !is_whitespace(ch) {
                break;
            }
            self.advance();
        }
    }
}

fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == ','
}

fn is_symbol_begin_char(ch: char) -> bool {
    !is_whitespace(ch)
        && match ch {
            '(' | ')' | '{' | '}' | '[' | ']' => false,
            _ => true,
        }
}

fn is_symbol_continue_char(ch: char) -> bool {
    !is_whitespace(ch)
        && match ch {
            '(' | ')' | '{' | '}' | '[' | ']' => false,
            _ => true,
        }
}

fn try_read_symbol_part<'i>(
    rdr: &mut CharReader<'i>,
) -> Result<Option<WithSpan<ByteIndexSpan, String>>, ReadError> {
    let (begin_byte_idx, first_ch) = rdr
        .current()
        .ok_or(ReadError::invalid_input((usize::MIN, usize::MAX)))?;
    if !is_symbol_begin_char(first_ch) {
        tracing::debug!("attempt to read symbol part beginning with non-'symbol begin' character, returning Ok(None)");
        return Ok(None);
        // return Err(ReadError::invalid_input(begin_byte_idx, begin_byte_idx));
    }

    let mut end_byte_idx = begin_byte_idx;

    tracing::trace!(
        "symbol part begins on {:?} with char {:?}",
        begin_byte_idx,
        first_ch
    );

    let mut buf = String::from(first_ch);
    rdr.advance();

    while let Some((ch_byte_idx, ch)) = rdr.current() {
        if ch == '/' || !is_symbol_continue_char(ch) {
            break;
        }
        buf.push(ch);
        end_byte_idx = ch_byte_idx;
        rdr.advance();
    }

    Ok(Some(WithSpan {
        data: buf,
        span: (begin_byte_idx, end_byte_idx),
    }))
}

impl<P: SharedPointerKind> ReadClj<P> for CharReader<'_> {
    fn try_read_one(&mut self) -> ReadResult<P> {
        while let Some((byte_idx, ch)) = self.current() {
            if is_whitespace(ch) {
                self.advance();
                continue;
            }

            match ch {
                // TODO: these can also occur when read-ing, e.g. [#_]
                ')' => {
                    // tracing::error!("prior value is unclosed");
                    self.advance();
                    return Err(ReadError::unclosed_collection((byte_idx, byte_idx)));
                }
                ']' => {
                    // tracing::error!("prior value is unclosed");
                    self.advance();
                    return Err(ReadError::unclosed_collection((byte_idx, byte_idx)));
                }
                '}' => {
                    // tracing::error!("prior value is unclosed");
                    self.advance();
                    return Err(ReadError::unclosed_collection((byte_idx, byte_idx)));
                }
                '(' => return self.try_read_list(),
                '[' => return self.try_read_vect(),
                '{' => return self.try_read_map(),
                '\'' => {
                    self.advance();
                    let SpanValue {
                        data: quoted,
                        span: (_, quoted_span_end),
                    } = match self.try_read_one()? {
                        Some(x) => x,
                        None => return Err(ReadError::InvalidInput((byte_idx, byte_idx))),
                    };
                    return Ok(Some(SpanValue {
                        data: Value::list_from_value_ptrs(vec![
                            ValuePtr::from(Value::Symbol(Symbol::unqualified(String::from(
                                "quote",
                            )))),
                            ValuePtr::from(quoted),
                        ]),
                        span: (byte_idx, quoted_span_end),
                    }));
                }
                '@' => {
                    self.advance();
                    let SpanValue {
                        data: derefed,
                        span,
                    } = match self.try_read_one()? {
                        Some(v) => v,
                        None => return Err(ReadError::InvalidInput((byte_idx, byte_idx))),
                    };
                    return Ok(Some(SpanValue {
                        data: Value::list_from_value_ptrs(vec![
                            ValuePtr::from(Value::Symbol(Symbol::unqualified(String::from(
                                "deref",
                            )))),
                            ValuePtr::from(derefed),
                        ]),
                        span: (byte_idx, span.1),
                    }));
                }
                '#' if self.peek_char_eq('{') => return self.try_read_set(),
                '#' if self.peek_char_eq(':') => {
                    self.advance(); // move beyond '#'
                    let keyword_result: ReadResult<P> = self.try_read_keyword();
                    let SpanValue {
                        data: _keyword_value, // TODO
                        span: keyword_span,
                    } = match keyword_result {
                        Ok(Some(span_value)) => span_value,
                        Ok(None) => {
                            return Err(ReadError::invalid_input((
                                byte_idx,
                                self.current_byte_idx().unwrap_or(0),
                            )));
                        }
                        Err(e) => return Err(e),
                    };
                    debug_assert!(self.current_char_eq('{'));
                    let map_begin_byte_idx = self.current_byte_idx().unwrap();
                    let map_result: ReadResult<P> = self.try_read_map();
                    let map_end_byte_idx = self.byte_idx_at(self.src_idx - 1).unwrap();
                    let SpanValue {
                        data: map_value,
                        span: map_span,
                    } = match map_result {
                        Ok(Some(span_value)) => span_value,
                        Ok(None) => {
                            return Err(ReadError::invalid_input((
                                map_begin_byte_idx,
                                map_end_byte_idx,
                            )))
                        }
                        Err(e) => return Err(e),
                    };
                    let span = (keyword_span.0, map_span.1);
                    //
                    // TODO: qualify unqualified top-level keyword keys in map, return updated map
                    //let map = map_value.into_map();
                    let data = map_value;
                    //
                    return Ok(Some(SpanValue { data, span }));
                }
                '#' if self.peek_char_eq('_') => {
                    self.advance(); // move beyond '#'
                    self.advance(); // move beyond '_'
                    self.skip_whitespaces();
                    let discarded: ReadResult<P> = self.try_read_one();
                    match discarded {
                        Ok(None) => {
                            return Err(ReadError::invalid_input((
                                byte_idx,
                                self.current_byte_idx().unwrap(),
                            )));
                        }
                        Err(ReadError::UnclosedCollection(byte_range)) => {
                            return Err(ReadError::unclosed_collection((byte_idx, byte_range.1)));
                        }
                        _ => {}
                    }
                    self.skip_whitespaces();
                }
                // todo: dispatch read
                // '#' if self.peek_char_eq('(') => { self.advance(); return self.try_read_list() },
                ':' => return self.try_read_keyword(),
                '"' => return self.try_read_string(),
                ';' => return self.try_read_comment(),
                ch if is_symbol_begin_char(ch) => {
                    let opt_span_value_symbol: Option<SpanValue<P>> = self.try_read_symbol()?;
                    let (symbol, symbol_span) = match opt_span_value_symbol {
                        Some(SpanValue { data, span }) => match data {
                            Value::Symbol(sym) => (sym, span),
                            _ => todo!(),
                        },
                        _ => todo!(),
                    };
                    return Ok(Some(SpanValue {
                        span: symbol_span,
                        data: match symbol {
                            Symbol::Unqualified { name } if name == "nil" => Value::Nil,
                            Symbol::Unqualified { name } if name == "true" => Value::Bool(true),
                            Symbol::Unqualified { name } if name == "false" => Value::Bool(false),
                            other => Value::Symbol(other),
                        },
                    }));
                }
                _ => todo!("{}", ch),
            }
        }

        Ok(None)
    }

    fn try_read_keyword(&mut self) -> ReadResult<P> {
        let (first_colon_byte_idx, first_colon_ch) = self
            .current()
            .ok_or(ReadError::invalid_input((usize::MIN, usize::MAX)))?;
        debug_assert_eq!(first_colon_ch, ':');
        self.advance();

        let double_colon = self.current_char_eq(':');
        if double_colon {
            self.advance();
        }

        let SpanValue::<P> {
            data: symbol,
            span: symbol_span,
        } = self.try_read_symbol()?.ok_or(ReadError::invalid_input((
            first_colon_byte_idx,
            first_colon_byte_idx,
        )))?;
        debug_assert!(symbol.is_symbol());
        let symbol = match symbol {
            Value::Symbol(sym) => sym,
            _ => {
                return Err(ReadError::invalid_input((
                    first_colon_byte_idx,
                    first_colon_byte_idx,
                )))
            }
        };

        let keyword = match (double_colon, symbol) {
            (false, Symbol::Unqualified { name }) => Keyword::unqualified(name),
            (false, Symbol::Qualified { namespace, name }) => Keyword::qualified(namespace, name),
            (true, Symbol::Unqualified { name }) => Keyword::self_qualified(name),
            (true, Symbol::Qualified { namespace, name }) => {
                Keyword::alias_qualified(namespace, name)
            }
        };

        Ok(Some(SpanValue {
            data: Value::Keyword(keyword),
            span: (first_colon_byte_idx, symbol_span.1),
        }))
    }

    fn try_read_string(&mut self) -> ReadResult<P> {
        let (str_begin_byte_idx, str_begin_ch) = self.current().expect("on string start");
        debug_assert_eq!(str_begin_ch, '"', "on string start");

        let mut str_end_byte_idx = None;

        tracing::trace!(
            "string begins on {:?} with char {:?}",
            str_begin_byte_idx,
            str_begin_ch,
        );

        let mut buf = String::new();
        self.advance(); // move beyond beginning '"'

        while let Some((byte_idx, ch)) = self.current() {
            if ch == '"' {
                str_end_byte_idx.replace(byte_idx);
                break;
            }
            buf.push(ch);
            self.advance();
        }

        match str_end_byte_idx {
            Some(str_end_byte_idx) => {
                tracing::trace!(
                    "string ends on {:?} with char {:?}",
                    self.current_byte_idx().unwrap(),
                    self.current_char().unwrap(),
                );
                self.advance(); // move beyond ending '"'

                Ok(Some(SpanValue {
                    data: Value::Str(buf),
                    span: (str_begin_byte_idx, str_end_byte_idx),
                }))
            }
            None => {
                tracing::error!("unclosed string literal");
                Err(ReadError::insufficient_input((
                    str_begin_byte_idx,
                    self.byte_idx_at(self.src_idx - 1).unwrap(),
                )))
            }
        }
    }

    fn try_read_symbol(&mut self) -> ReadResult<P> {
        let WithSpan {
            span: part1_span,
            data: part1_buf,
        } = match try_read_symbol_part(self)? {
            Some(part1) => part1,
            None => {
                tracing::warn!("called try_read_symbol_v2 with invalid CharReader state");
                return Ok(None);
            }
        };

        if part1_buf.len() > "/".len() && part1_buf.starts_with('/') {
            tracing::debug!("invalid symbol form (leading slash)");
            return Err(ReadError::invalid_input(part1_span));
        }

        let is_qualified = self.current_char_eq('/');

        if is_qualified {
            if self.is_at_end() {
                return Err(ReadError::invalid_input((
                    part1_span.0,
                    self.current_byte_idx().unwrap(),
                )));
            }

            self.advance(); // beyond '/'

            let WithSpan {
                span: part2_span,
                data: part2_buf,
            } = match try_read_symbol_part(self)? {
                Some(part2) => part2,
                None => {
                    tracing::warn!("called try_read_symbol_v2 with invalid CharReader state");
                    return Ok(None);
                }
            };

            if part2_buf.len() > "/".len() && part2_buf.starts_with('/') {
                tracing::debug!("invalid symbol form (leading slash)");
                return Err(ReadError::invalid_input(part2_span));
            }

            Ok(Some(SpanValue {
                data: Value::Symbol(Symbol::qualified(part1_buf, part2_buf)),
                span: (part1_span.0, part2_span.1),
            }))
        } else {
            Ok(Some(SpanValue {
                data: Value::Symbol(Symbol::unqualified(part1_buf)),
                span: part1_span,
            }))
        }
    }

    fn try_read_list(&mut self) -> ReadResult<P> {
        let (list_begin_byte_idx, list_begin_ch) = self.current().expect("on list start");
        debug_assert_eq!(list_begin_ch, '(', "on list start");

        tracing::trace!(
            "list begins on {:?} with char {:?}",
            list_begin_byte_idx,
            list_begin_ch,
        );

        let mut value_ptrs = vec![];

        self.advance(); // move beyond '('

        loop {
            self.skip_whitespaces();

            if self.is_beyond_end() {
                tracing::error!("is beyond end while reading list");
                return Err(ReadError::insufficient_input((
                    list_begin_byte_idx,
                    self.byte_idx_at(self.src.len() - 1).unwrap(),
                )));
            }

            if let Some((byte_idx, ')')) = self.current() {
                self.advance(); // move beyond )
                return Ok(Some(SpanValue {
                    span: (list_begin_byte_idx, byte_idx),
                    data: Value::list_from_value_ptrs(value_ptrs),
                }));
            }

            match self.try_read_one()? {
                Some(SpanValue { data: value, .. }) => {
                    value_ptrs.push(ValuePtr::from(value));

                    self.skip_whitespaces();

                    if let Some((byte_idx, ')')) = self.current() {
                        self.advance(); // move beyond )
                        return Ok(Some(SpanValue {
                            span: (list_begin_byte_idx, byte_idx),
                            data: Value::list_from_value_ptrs(value_ptrs),
                        }));
                    }
                }
                None => todo!("when does this happen"),
            }
        }
    }

    fn try_read_vect(&mut self) -> ReadResult<P> {
        let (vect_begin_byte_idx, vect_begin_ch) = self.current().expect("on vect start");
        debug_assert_eq!(vect_begin_ch, '[', "on vect start");

        let mut vect_end_byte_idx = vect_begin_byte_idx;

        let mut value_ptrs = vec![];

        self.advance();

        loop {
            self.skip_whitespaces();

            if self.is_beyond_end() {
                return Err(ReadError::insufficient_input((
                    vect_begin_byte_idx,
                    self.byte_idx_at(self.src_idx - 1).unwrap(),
                )));
            }

            if let Some((byte_idx, ']')) = self.current() {
                vect_end_byte_idx = byte_idx;
                self.advance();
                return Ok(Some(SpanValue {
                    span: (vect_begin_byte_idx, vect_end_byte_idx),
                    data: Value::vect_from_value_ptrs(value_ptrs),
                }));
            }

            match self.try_read_one()? {
                Some(SpanValue { data: value, .. }) => {
                    value_ptrs.push(ValuePtr::from(value));

                    self.skip_whitespaces();

                    if let Some((byte_idx, ']')) = self.current() {
                        vect_end_byte_idx = byte_idx;
                        self.advance();
                        return Ok(Some(SpanValue {
                            span: (vect_begin_byte_idx, vect_end_byte_idx),
                            data: Value::vect_from_value_ptrs(value_ptrs),
                        }));
                    }
                }
                None => match self.current() {
                    Some((byte_idx, ']')) => {
                        vect_end_byte_idx = byte_idx;
                        self.advance();
                        return Ok(Some(SpanValue {
                            span: (vect_begin_byte_idx, vect_end_byte_idx),
                            data: Value::vect_from_value_ptrs(value_ptrs),
                        }));
                    }
                    _ => {
                        return Err(ReadError::insufficient_input((
                            vect_begin_byte_idx,
                            vect_end_byte_idx,
                        )));
                    }
                },
            }
        }
    }

    fn try_read_set(&mut self) -> ReadResult<P> {
        let (set_begin_byte_idx, set_begin_ch) = self.current().expect("on set start");
        debug_assert_eq!(set_begin_ch, '#', "on set start");
        debug_assert!(self.peek_char_eq('{'), "correct set start");

        tracing::trace!(
            "set begins on {:?} with char {:?}",
            set_begin_byte_idx,
            set_begin_ch,
        );

        let mut set_end_byte_idx = set_begin_byte_idx;

        let mut value_ptrs = vec![];

        self.advance(); // #
        self.advance(); // {

        loop {
            self.skip_whitespaces();

            if self.is_beyond_end() {
                tracing::error!("insufficient set input");
                return Err(ReadError::insufficient_input((
                    set_begin_byte_idx,
                    self.byte_idx_at(self.src_idx - 1).unwrap(),
                )));
            }

            if let Some((byte_idx, '}')) = self.current() {
                set_end_byte_idx = byte_idx;
                self.advance();
                return Ok(Some(SpanValue {
                    span: (set_begin_byte_idx, set_end_byte_idx),
                    data: Value::set_from_value_ptrs(value_ptrs),
                }));
            }

            match self.try_read_one()? {
                Some(SpanValue { data: value, .. }) => {
                    value_ptrs.push(ValuePtr::from(value));

                    self.skip_whitespaces();

                    if let Some((byte_idx, '}')) = self.current() {
                        set_end_byte_idx = byte_idx;
                        self.advance();
                        return Ok(Some(SpanValue {
                            span: (set_begin_byte_idx, set_end_byte_idx),
                            data: Value::set_from_value_ptrs(value_ptrs),
                        }));
                    }
                }
                None => match self.current() {
                    Some((byte_idx, '}')) => {
                        set_end_byte_idx = byte_idx;
                        self.advance();
                        return Ok(Some(SpanValue {
                            span: (set_begin_byte_idx, set_end_byte_idx),
                            data: Value::set_from_value_ptrs(value_ptrs),
                        }));
                    }
                    _ => {
                        return Err(ReadError::insufficient_input((
                            set_begin_byte_idx,
                            set_end_byte_idx,
                        )));
                    }
                },
            }
        }
    }

    fn try_read_map(&mut self) -> ReadResult<P> {
        let (map_begin_byte_idx, map_begin_ch) = self.current().expect("on map start");
        debug_assert_eq!(map_begin_ch, '{', "on map start");

        let mut value_ptr_pairs = vec![];

        self.advance(); // move beyond '{'

        loop {
            self.skip_whitespaces();

            if self.is_beyond_end() {
                tracing::error!("unclosed map literal");
                return Err(ReadError::insufficient_input((
                    map_begin_byte_idx,
                    self.byte_idx_at(self.src_idx - 1).unwrap(),
                )));
            }

            match self.current() {
                Some((byte_idx, '}')) => {
                    self.advance(); // move beyond '}'
                    return Ok(Some(SpanValue {
                        span: (map_begin_byte_idx, byte_idx),
                        data: Value::map_from_value_ptr_pairs(value_ptr_pairs),
                    }));
                }
                None => {
                    tracing::error!("unclosed map literal");
                    return Err(ReadError::insufficient_input((
                        map_begin_byte_idx,
                        self.byte_idx_at(self.src_idx - 1).unwrap(),
                    )));
                }
                _ => { /* carry-on to read-ing key */ }
            }

            let key = match self.try_read_one()? {
                Some(SpanValue { data: key, .. }) => key,
                None => {
                    self.skip_whitespaces();

                    if let Some((byte_idx, '}')) = self.current() {
                        self.advance(); // move beyond '}'
                        return Ok(Some(SpanValue {
                            span: (map_begin_byte_idx, byte_idx),
                            data: Value::map_from_value_ptr_pairs(value_ptr_pairs),
                        }));
                    }

                    tracing::error!("map: no key found");
                    return Err(ReadError::insufficient_input((
                        map_begin_byte_idx,
                        self.current_byte_idx().unwrap(),
                    )));
                }
            };

            self.skip_whitespaces();

            if let Some((byte_idx, '}')) = self.current() {
                self.advance(); // move beyond '}'
                tracing::error!("map: no value for key found");
                return Err(ReadError::insufficient_input((
                    // value-less key
                    map_begin_byte_idx,
                    byte_idx,
                )));
            }

            let value = match self.try_read_one()? {
                Some(SpanValue { data: value, .. }) => value,
                None => {
                    self.skip_whitespaces();

                    match self.current() {
                        Some((byte_idx, '}')) => {
                            self.advance(); // move beyond '}'
                            tracing::error!("map: no value for key found");
                            return Err(ReadError::insufficient_input((
                                map_begin_byte_idx,
                                byte_idx,
                            )));
                        }
                        Some(_) => unreachable!(),
                        None => {
                            tracing::error!("unclosed & unabalanced map literal");
                            return Err(ReadError::insufficient_input((
                                map_begin_byte_idx,
                                self.byte_idx_at(self.src_idx - 1).unwrap(),
                            )));
                        }
                    }
                }
            };

            value_ptr_pairs.push((ValuePtr::from(key), ValuePtr::from(value)));
        }
    }

    fn try_read_comment(&mut self) -> ReadResult<P> {
        let (_comment_begin_byte_idx, comment_begin_ch) = self.current().expect("on comment start");
        debug_assert_eq!(comment_begin_ch, ';', "on comment start");
        self.advance(); // move beyond beginning ';'
        while let Some(ch) = self.current_char() {
            match ch {
                '\n' | '\r' => break,
                _ => {}
            }
            self.advance();
        }
        Ok(None)
    }
}

// in these tests Rc vs Arc is incidental, had to make some choice
#[cfg(test)]
mod t {
    use crate::{
            char_reader::{self, is_whitespace},
            keyword::Keyword,
            reader,
            value::{RcValue, Value, ValuePtr},
            ReadClj, ReadError, ReadResult, SpanValue,
    };
    use cljrs_core::symbol::Symbol;

    use super::CharReader;

    #[test]
    fn single_forward_slash_symbol() {
        let mut rdr = reader("/").unwrap();
        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(v, RcValue::Symbol(Symbol::unqualified(String::from("/"))));
    }

    #[test]
    fn double_forward_slash_symbol() {
        let mut rdr = reader("//").unwrap();
        let v: ReadResult<archery::RcK> = rdr.try_read_one();
        assert_eq!(v.unwrap_err(), ReadError::invalid_input((0, 1)));
    }

    #[test]
    fn unqualified_symbol() {
        let mut rdr = reader("assoc").unwrap();
        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(
            v,
            RcValue::Symbol(Symbol::unqualified(String::from("assoc")))
        );
    }

    #[test]
    fn qualified_symbol() {
        let mut rdr = reader("foo/bar").unwrap();
        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(
            v,
            RcValue::Symbol(Symbol::qualified(String::from("foo"), String::from("bar")))
        );
    }

    #[test]
    fn empty_list() {
        let mut rdr = reader("()").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue { data: value, .. }| value)),
            Ok(Some(RcValue::empty_list())),
        );
    }

    #[test]
    fn simple_list() {
        let mut rdr = CharReader::try_from_str("(assoc)").unwrap();
        assert_eq!(rdr.current_byte_idx().unwrap(), 0);

        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;

        assert!(rdr.current_char().is_none());
        assert!(rdr.is_beyond_end());

        assert_eq!(
            v,
            RcValue::list_from_value_ptrs(vec![ValuePtr::from(RcValue::Symbol(
                Symbol::unqualified(String::from("assoc"))
            )),])
        );
    }

    #[test]
    fn empty_vect() {
        let mut rdr = reader("[]").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue { data: value, .. }| value)),
            Ok(Some(RcValue::empty_vect())),
        );
    }

    #[test]
    fn simple_vect() {
        let mut rdr = CharReader::try_from_str("[assoc]").unwrap();
        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;
        assert!(rdr.is_beyond_end());
        assert!(rdr.current().is_none());
        assert_eq!(
            v,
            RcValue::vect_from_value_ptrs(vec![ValuePtr::from(RcValue::Symbol(
                Symbol::unqualified(String::from("assoc"))
            )),])
        );
    }

    #[test]
    fn empty_set() {
        let mut rdr = CharReader::try_from_str("#{}").unwrap();
        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(v, RcValue::empty_set());
    }

    #[test]
    fn simple_set() {
        let mut rdr = CharReader::try_from_str("#{assoc}").unwrap();
        let v: RcValue = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(
            v,
            RcValue::set_from_value_ptrs(vec![ValuePtr::from(RcValue::Symbol(
                Symbol::unqualified(String::from("assoc"))
            )),])
        );
    }

    #[test]
    fn empty_map() {
        let mut rdr = reader("{}").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue { data: value, .. }| value)),
            Ok(Some(RcValue::empty_map())),
        );
    }

    #[test]
    fn simple_map() {
        let mut rdr = reader("{k v}").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue { data: value, .. }| value)),
            Ok(Some(RcValue::map_from_value_ptr_pairs(vec![(
                ValuePtr::from(RcValue::Symbol(Symbol::unqualified(String::from("k")))),
                ValuePtr::from(RcValue::Symbol(Symbol::unqualified(String::from("v"))))
            )]))),
        );
    }

    #[test]
    fn unbalanced_map() {
        let mut rdr = reader("{k}").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue::<archery::RcK> { data: value, .. }| value)),
            Err(ReadError::insufficient_input((0, 2))),
        );
    }

    #[test]
    fn unbalanced_map_2() {
        let src = "{k v, x}";
        let mut rdr: Box<dyn ReadClj<archery::RcK>> = reader(src).unwrap();
        let res = rdr.try_read_one().unwrap_err();
        assert_eq!(res, ReadError::insufficient_input((0, 7)),);
    }

    #[test]
    fn unclosed_unbalanced_map() {
        let src = "{k v, x";
        let mut rdr: Box<dyn ReadClj<archery::RcK>> = reader(src).unwrap();
        let res = rdr.try_read_one().unwrap_err();
        assert_eq!(res, ReadError::insufficient_input((0, 6)),);
    }

    #[test]
    fn given_currently_on_whitespace_when_skip_whitespaces_called_then_advances_beyond_whitespace()
    {
        let src = "(hello) (world)";
        // ....... 0123456789...
        let mut rdr = CharReader::try_from_str(src).unwrap();
        assert_eq!(rdr.current_byte_idx().unwrap(), 0);
        for _ in 0..7 {
            rdr.advance();
        }
        assert!(is_whitespace(rdr.current_char().unwrap()));
        rdr.skip_whitespaces();
        assert!(!is_whitespace(rdr.current_char().unwrap()));
        assert_eq!(rdr.current_byte_idx().unwrap(), 8);
    }

    #[test]
    fn is_beyond_end() {
        let mut rdr = CharReader::try_from_str("(assoc)").unwrap();
        let _res: ReadResult<archery::RcK> = rdr.try_read_one();

        assert!(rdr.current_char().is_none());
        assert!(rdr.is_beyond_end());
    }

    #[test]
    fn empty_set_in_list() {
        let mut rdr = reader("(#{})").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue { data: value, .. }| value)),
            Ok(Some(RcValue::list_from_value_ptrs(vec![ValuePtr::from(
                RcValue::empty_set()
            )]))),
        );
    }

    #[test]
    fn simple_set_in_list() {
        let mut rdr = reader("(#{assoc})").unwrap();
        assert_eq!(
            rdr.try_read_one()
                .map(|opt| opt.map(|SpanValue { data: value, .. }| value)),
            Ok(Some(RcValue::list_from_value_ptrs(vec![
                RcValue::set_ptr_from_value_ptrs(vec![ValuePtr::from(RcValue::Symbol(
                    Symbol::unqualified(String::from("assoc"))
                ))])
            ]))),
        );
    }

    #[test]
    fn sequential_reads() {
        let mut rdr = reader("hello world").unwrap();
        let r1 = rdr.try_read_one().unwrap().unwrap().data;
        let r2 = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(
            r1,
            RcValue::Symbol(Symbol::unqualified(String::from("hello")))
        );
        assert_eq!(
            r2,
            RcValue::Symbol(Symbol::unqualified(String::from("world")))
        );
    }

    #[test]
    fn leading_forward_slash_invalid_symbol() {
        let src = "/assoc";
        let mut rdr = reader(src).unwrap();
        let res: ReadResult<archery::RcK> = rdr.try_read_one();
        let err = res.expect_err("invalid symbol form (leading slash)");
        assert!(err.is_invalid_input());
    }

    #[test]
    fn trailing_forward_slash_invalid_unqualified_symbol() {
        let src = "assoc/";
        let mut rdr = reader(src).unwrap();
        let res: ReadResult<archery::RcK> = rdr.try_read_one();
        let err = res.expect_err("invalid symbol form (trailing slash)");
        assert!(err.is_invalid_input());
    }

    #[test]
    fn trailing_forward_slash_valid_qualified_symbol() {
        let src = "clojure.core//";
        let mut rdr = reader(src).unwrap();
        let value = rdr.try_read_one().unwrap().unwrap().data;
        assert_eq!(
            value,
            RcValue::Symbol(Symbol::qualified(
                String::from("clojure.core"),
                String::from("/")
            ))
        )
    }

    #[ignore = "todo: rewrite part of reader to do this"]
    #[test]
    fn too_many_trailing_forward_slashes_invalid_qualified_symbol() {
        let src = "clojure.core///";
        let mut rdr = reader(src).unwrap();
        let res: ReadResult<archery::RcK> = rdr.try_read_one();
        let err = res.expect_err("invalid symbol form (trailing slash)");
        assert!(err.is_invalid_input());
    }

    #[test]
    fn symbol_part() {
        let src = "foo";
        let mut rdr = CharReader::try_from_str(src).unwrap();
        let res = char_reader::try_read_symbol_part(&mut rdr);
        let value = res.unwrap().unwrap().data;
        assert_eq!(value, String::from("foo"));
    }

    #[test]
    fn composite_symbol_parts() {
        let src = "foo/bar";
        let mut rdr = CharReader::try_from_str(src).unwrap();
        let sym_part_1 = char_reader::try_read_symbol_part(&mut rdr)
            .expect("read-able symbol part")
            .expect("non-empty")
            .data;
        assert_eq!(sym_part_1, String::from("foo"));
        rdr.advance();
        let sym_part_2 = char_reader::try_read_symbol_part(&mut rdr)
            .expect("read-able symbol part")
            .expect("non-empty")
            .data;
        assert_eq!(sym_part_2, String::from("bar"));
    }

    #[test]
    fn quoted_returns_list() {
        let src = "'foo";
        let mut rdr = reader(src).unwrap();
        let quoted = rdr
            .try_read_one()
            .expect("read-able quoted symbol")
            .expect("non-empty")
            .data;
        assert_eq!(
            quoted,
            RcValue::list_from_value_ptrs(vec![
                ValuePtr::from(Value::Symbol(Symbol::unqualified(String::from("quote")))),
                ValuePtr::from(Value::Symbol(Symbol::unqualified(String::from("foo")))),
            ])
        );
    }

    #[test]
    fn unqualified_keyword() {
        let src = ":foo";
        let mut rdr = reader(src).unwrap();
        let value = rdr
            .try_read_one()
            .expect("read-able unqualified keyword")
            .expect("non-empty")
            .data;
        assert_eq!(
            value,
            RcValue::Keyword(Keyword::unqualified(String::from("foo")))
        );
    }

    #[test]
    fn qualified_keyword() {
        let src = ":foo/bar";
        let mut rdr = reader(src).unwrap();
        let value = rdr
            .try_read_one()
            .expect("read-able qualified keyword")
            .expect("non-empty")
            .data;
        assert_eq!(
            value,
            RcValue::Keyword(Keyword::qualified(String::from("foo"), String::from("bar")))
        );
    }

    #[test]
    fn self_qualified_keyword() {
        let src = "::foo";
        let mut rdr = reader(src).unwrap();
        let value = rdr
            .try_read_one()
            .expect("read-able self-qualified keyword")
            .expect("non-empty")
            .data;
        assert_eq!(
            value,
            RcValue::Keyword(Keyword::self_qualified(String::from("foo")))
        );
    }

    #[test]
    fn alias_qualified_keyword() {
        let src = "::foo/bar";
        let mut rdr = reader(src).unwrap();
        let value = rdr
            .try_read_one()
            .expect("read-able alias-qualified keyword")
            .expect("non-empty")
            .data;
        assert_eq!(
            value,
            RcValue::Keyword(Keyword::alias_qualified(
                String::from("foo"),
                String::from("bar")
            ))
        );
    }

    #[test]
    fn simple_string() {
        let src = "\"hello\"";
        let mut rdr = reader(src).unwrap();
        let value = rdr
            .try_read_one()
            .expect("read-able string")
            .expect("non-empty")
            .data;
        assert_eq!(value, RcValue::Str(String::from("hello")),);
    }

    #[test]
    fn symbol_ends_at_newline() {
        let src = "x\ny";
        let mut rdr = reader(src).unwrap();
        let mut values = vec![];
        loop {
            match rdr.try_read_one() {
                Ok(Some(SpanValue { data, .. })) => {
                    values.push(data);
                }
                Ok(None) => break,
                Err(err) => eprintln!("{:?}", err),
            }
        }
        assert_eq!(
            values,
            vec![
                RcValue::Symbol(Symbol::unqualified(String::from("x"))),
                RcValue::Symbol(Symbol::unqualified(String::from("y"))),
            ]
        );
    }

    #[test]
    fn comment_on_own_line() {
        let src = "hello\n; world\nbob";
        let mut rdr = reader(src).unwrap();
        let mut values = vec![];
        while let Ok(opt_span_val) = rdr.try_read_one() {
            match opt_span_val {
                Some(SpanValue { data, .. }) => values.push(data),
                None => {}
            }
        }
        assert_eq!(
            values,
            vec![
                RcValue::Symbol(Symbol::unqualified(String::from("hello"))),
                RcValue::Symbol(Symbol::unqualified(String::from("bob"))),
            ]
        );
    }

    #[test]
    fn qualified_map() {
        let src = "#:foo{:bar :zap}";
        let mut rdr = reader(src).unwrap();
        let SpanValue { data: val, .. } = rdr.try_read_one().unwrap().unwrap();
        assert_eq!(
            val,
            RcValue::map_from_value_pairs(vec![(
                Value::qualified_keyword(String::from("foo"), String::from("bar")),
                Value::unqualified_keyword(String::from("zap")),
            )]),
        );
    }
}
