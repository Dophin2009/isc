#[derive(Debug, Clone)]
pub struct Grammar<T, N>
where
    T: Clone + PartialEq,
    N: Clone + PartialEq,
{
    pub productions: Vec<Production<T, N>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Production<T, N>
where
    T: Clone + PartialEq,
    N: Clone + PartialEq,
{
    pub head: N,
    pub body: Vec<Symbol<T, N>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol<T, N>
where
    T: Clone + PartialEq,
    N: Clone + PartialEq,
{
    Terminal(T),
    Nonterminal(N),
}

impl<T, N> PartialEq<N> for Symbol<T, N>
where
    T: Clone + PartialEq,
    N: Clone + PartialEq,
{
    fn eq(&self, other: &N) -> bool {
        match self {
            Self::Terminal(_) => false,
            Self::Nonterminal(n) => n == other,
        }
    }
}

impl<T, N> Grammar<T, N>
where
    T: Clone + PartialEq,
    N: Clone + PartialEq,
{
    pub fn productions_with_head(&self, head: &N) -> Vec<&Production<T, N>> {
        self.productions
            .iter()
            .filter(|&p| p.head == *head)
            .collect()
    }
}
