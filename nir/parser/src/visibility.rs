use crate::{Parse, ParseInput, Symbol};

use ast::{keywords::Pub, Span, Visibility, VisibilityKind};

impl<I> Parse<I> for Visibility
where
    I: Iterator<Item = Symbol>,
{
    #[inline]
    fn parse(input: &mut ParseInput<I>) -> Result<Self, ()> {
        let (kind, span) = match input.peek() {
            Some(peeked) => match peeked.0 {
                reserved!(Pub) => {
                    let spanned = input.consume::<Pub>()?;
                    (VisibilityKind::Public, spanned.1)
                }
                _ => {
                    let pos = input.last_pos();
                    (VisibilityKind::Private, Span::new(pos, pos))
                }
            },
            None => {
                let pos = input.last_pos();
                (VisibilityKind::Private, Span::new(pos, pos))
            }
        };

        Ok(Visibility { kind, span })
    }
}
