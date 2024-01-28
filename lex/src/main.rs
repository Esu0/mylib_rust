struct Tokenizer<C: ?Sized> {
    buffer: Option<u8>,
    chars: C,
}

trait Chars {
    fn next_char(&mut self) -> Option<u8>;

    fn into_tokenizer(mut self) -> Tokenizer<Self>
    where
        Self: Sized,
    {
        Tokenizer {
            buffer: self.next_char(),
            chars: self,
        }
    }
}

impl<C: ?Sized + Chars> Tokenizer<C> {
    fn next_char(&mut self) -> Option<u8> {
        std::mem::replace(&mut self.buffer, self.chars.next_char())
    }

    fn peek(&self) -> Option<u8> {
        self.buffer
    }

    fn expect<T: TokenKind + Eq>(&mut self, kind: T) -> T::Token {
        if T::predict(self.peek().unwrap()) !=  kind {
            panic!()
        }
        T::tokenize(self).unwrap()
    }

    fn consume<T: TokenKind + Eq>(&mut self, kind: T) -> Option<T::Token> {
        self.peek().and_then(|c| {
            if T::predict(c) == kind {
                T::tokenize(self)
            } else {
                None
            }
        })
    }
}

trait TokenKind {
    type Token;
    fn predict(c: u8) -> Self;

    fn tokenize<C: ?Sized + Chars>(tokenizer: &mut Tokenizer<C>) -> Option<Self::Token>;

    fn kind(token: &Self::Token) -> Self;
}

enum Test {
    Identifier
}

impl TokenKind for Test {
    type Token = String;
    fn predict(_c: u8) -> Self
    where
        Self: Sized,
    {
        Self::Identifier
    }

    fn tokenize<C: ?Sized + Chars>(_tokenizer: &mut Tokenizer<C>) -> Option<Self::Token> {
        Some("abc".to_owned())
    }

    fn kind(_token: &Self::Token) -> Self {
        Self::Identifier
    }
}

struct TestTokenizer;

impl Chars for TestTokenizer {
    fn next_char(&mut self) -> Option<u8> {
        unimplemented!()
    }
}

fn main() {
    println!("Hello, world!");
}
