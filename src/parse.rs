use std::{collections::HashMap, fmt::Display};

pub struct Story {
    map: HashMap<SectionIdentifier, Section>,
}

impl Story {
    pub fn parse(story: &str) -> ParseResult<Story> {
        // TODO: attempt to, in a loop, parse as a section --- error if failure

        let mut map: HashMap<SectionIdentifier, Section> = HashMap::new();

        let mut iter = story.lines().peekable();

        loop {
            if iter.peek().is_none() {
                // we're out of lines --- running out in the middle of a section is
                // an error, but here, it just means there aren't any more sections
                break;
            }

            let section = Section::parse(&mut iter)?;

            if let Some(dup_section) = map.get(&section.identifier) {
                return Err(ParseError::DuplicateSections(section, dup_section.clone()));
            } else {
                map.insert(section.identifier.clone(), section)
                    // we want to panic if Some gets returned
                    .map_or(Ok(()), |_| Err(()))
                    .expect("verified above");
            }
        }

        // TODO: check that a Start key exists,

        Ok(Self { map })
    }
}

#[derive(Clone, Debug)]
struct Section {
    identifier: SectionIdentifier,
    description: Description,
    choices: Vec<Choice>,
}

impl Section {
    fn parse<'a>(mut iter: impl Iterator<Item = &'a str>) -> ParseResult<Self> {
        let section_identifier =
            SectionIdentifier::parse(iter.next().expect("required to be Some"), true)?;

        let mut section_description = String::new();
        let mut reached_choices = false;
        let mut choices = Vec::new();
        for line in iter {
            // empty lines are fine
            if line.trim().is_empty() {
                continue;
            }

            if line.trim() == "---" {
                return Err(ParseError::UnexpectedSeparator);
            }

            let Ok(choice) = Choice::parse(line) else {
                // if we find something that isn't a choice, after we've gotten
                // to choices, then that's an error
                if !reached_choices {
                    section_description.push_str(line);
                    continue;
                } else {
                    return Err(ParseError::ExpectedChoice);
                }
            };
            // if we've reached this far, we've successfully parsed it as a
            // choice
            reached_choices = true;
            choices.push(choice);
        }

        // if we end without finding any description, that's an error, and
        // idem for choices
        if section_description.is_empty() {
            return Err(ParseError::ExpectedDescription);
        } else if choices.is_empty() {
            return Err(ParseError::ExpectedChoice);
        }

        let section_description = Description::parse(&section_description);

        Ok(Self {
            identifier: section_identifier,
            description: section_description,
            choices,
        })
    }
}

#[derive(Clone, Debug)]
struct Choice {
    description: Description,
    goto: SectionIdentifier,
}

impl Choice {
    fn parse(str: &str) -> ParseResult<Self> {
        // we want to parse it as the following: any characters, until we get
        // to an ->, and then a section identifier
        // any requirements on the description? pretty much nothing

        let parts: Vec<&str> = str.split("->").collect();

        if (parts.len() < 2) || parts[0].ends_with('\\') {
            return Err(ParseError::MissingArrow);
        } else if parts.len() > 2 {
            return Err(ParseError::MultipleArrows);
        }

        let goto = SectionIdentifier::parse(parts[1], false)?;
        let description = Description::parse(parts[0]);

        Ok(Self { description, goto })
    }
}

// just a newtype, that's all caps, numeric, and underscores
// must have *some* alphabetic. can't be all underscores/numeric
#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct SectionIdentifier(String);

impl SectionIdentifier {
    // def is whether it's a definition (in which case we want a final colon),
    // or not, in which case we don't
    fn parse(str: &str, def: bool) -> ParseResult<Self> {
        let mut found_alphabetic = false;
        let mut colon = false;

        for ch in str.trim().chars() {
            if ((ch.is_ascii_alphabetic() && ch.is_ascii_uppercase())
                || ch.is_ascii_digit()
                || ch == '_')
                && !colon
            // there should only ever be one colon, and that at the end ---
            // if it's true, it means we've found two, and is thus an error
            {
                if ch.is_ascii_alphabetic() {
                    found_alphabetic = true;
                }
            } else if ch == ':' && !colon {
                colon = true;
            } else {
                return Err(ParseError::SectionIdentifier(ch));
            }
        }

        if def && !colon {
            Err(ParseError::MissingColon)
        } else if !found_alphabetic {
            Err(ParseError::MissingAlphabetic)
        } else {
            Ok(Self(str.to_owned()))
        }
    }
}

#[derive(Clone, Debug)]
struct Description(Vec<Paragraph>);

impl Description {
    fn parse(str: &str) -> Self {
        // TODO: all we're doing is: parsing into sections
        // then each section eliminates newlines

        let str: String = str
            .lines()
            .map(|line| format!("{}\n", line.trim()))
            .collect();

        let desc = str
            .split("\n\n")
            .map(|paragraph| Paragraph::parse(paragraph))
            .collect();

        Self(desc)
    }
}

#[derive(Clone, Debug)]
struct Paragraph(String);

impl Paragraph {
    fn parse(str: &str) -> Self {
        Self(str.replace('\n', " "))
    }
}

type ParseResult<T> = Result<T, ParseError>;

// #[derive(Clone, Debug)]
// pub struct ParseError {
//     error_type: ParseErrorType,
// }

// impl Display for ParseError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.error_type)
//     }
// }

// impl Error for ParseError {}

#[derive(Clone, Debug)]
pub enum ParseError {
    SectionIdentifier(char),
    MissingColon,
    MissingAlphabetic,
    UnexpectedSeparator,
    MissingArrow,
    MultipleArrows,
    ExpectedChoice,
    ExpectedDescription,
    DuplicateSections(Section, Section),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
        //     ParseError::Foo => write!(f, "FOO"),
        // }
        todo!()
    }
}
