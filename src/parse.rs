use std::{
    collections::HashMap, error::Error, fmt::Display, fs, io, num::TryFromIntError, path::Path,
};

use wasm_bindgen::prelude::wasm_bindgen;

pub fn load_file(path: impl AsRef<Path>) -> io::Result<ParseResult<Story>> {
    Ok(Story::parse(&fs::read_to_string(path)?))
}

pub fn load_str(str: &str) -> ParseResult<Story> {
    Story::parse(str)
}

// #[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Story {
    sections: HashMap<SectionIdentifier, Section>,
}

// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Story {
//     sections: HashMap<SectionIdentifier, Section>,
// }

impl Story {
    pub fn parse(story: &str) -> ParseResult<Story> {
        // TODO: attempt to, in a loop, parse as a section --- error if failure

        let mut sections: HashMap<SectionIdentifier, Section> = HashMap::new();

        // TODO: trim before the lines iter?
        let mut iter = story
            .lines()
            .enumerate()
            .filter(|(_, str)| !str.starts_with('#'))
            .peekable();

        loop {
            if iter.peek().is_none() {
                // we're out of lines --- running out in the middle of a section is
                // an error, but here, it just means there aren't any more sections
                break;
            } else if iter.peek().expect("not none, as above").1.is_empty() {
                iter.next();
                continue;
            }

            let (line_num, text) = iter.peek().expect("validated above").to_owned();
            let section = Section::parse(&mut iter)?;

            if let Some(dup_section) = sections.get(&section.identifier) {
                return Err(ParseError::new(
                    ParseErrorType::DuplicateSections(section, dup_section.clone()),
                    line_num,
                    text.to_owned(),
                ));
            }
            sections
                .insert(section.identifier.clone(), section)
                // we want to panic if Some gets returned
                .map_or(Ok(()), |_| Err(()))
                .expect("verified above");
        }

        for section in sections.values() {
            for choice in &section.choices {
                if !matches!(choice.goto.0.as_str(), "__RESTART" | "__MENU")
                    && !sections.contains_key(&choice.goto)
                {
                    return Err(ParseError::new(
                        ParseErrorType::DanglingGoto,
                        section.line_num,
                        format!("{} -> {}", choice.description, choice.goto),
                    ));
                }
            }
        }

        Ok(Self { sections })
    }

    pub fn sections(&self) -> &HashMap<SectionIdentifier, Section> {
        &self.sections
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Section {
    identifier: SectionIdentifier,
    description: Description,
    choices: Vec<Choice>,
    // TODO: how to avoid this being necessary when when validating?
    // this hoists, obviously, so we can't just validate as we parse
    line_num: usize,
}

impl Section {
    fn parse<'a>(mut iter: impl Iterator<Item = (usize, &'a str)>) -> ParseResult<Self> {
        // let section_identifier = { for line in iter {
        //     if line
        // } };

        let mut section_identifier =
            SectionIdentifier::parse(iter.next().expect("required to be Some"), true)?;

        let mut section_description = String::new();
        let mut reached_choices = false;
        let mut finished = false;
        let mut choices = Vec::new();
        let mut line_num = None;
        let mut current_line = None;
        for (num, mut line) in iter {
            // if num > 195 {
            //     dbg!(&num);
            //     dbg!(&line);
            //     dbg!(&choices);
            //     dbg!(&section_description);
            // }

            line_num = Some(num);
            current_line = Some(line);
            line = line.trim();

            // ignore it, unless it's part of the description, because then
            // it's meaningfull and should be included
            if line.is_empty() && reached_choices {
                continue;
            }

            if line == "---" {
                if reached_choices {
                    finished = true;
                    break;
                } else {
                    return Err(ParseError::new(
                        // TODO: which do we want?
                        // ParseErrorType::UnexpectedSeparator,
                        ParseErrorType::ExpectedChoice,
                        num,
                        line.to_owned(),
                    ));
                }
            }

            let choice = match Choice::parse(line, num) {
                Ok(choice) => choice,
                Err(err) => {
                    if !reached_choices {
                        section_description.push_str(&format!("{}\n", line));
                        continue;
                    } else {
                        // return Err(ParseError::new(ParseErrorType::ExpectedChoice, line_num));
                        return Err(err);
                    }
                }
            };
            // let Ok(choice) = Choice::parse(line, line_num) else {
            //     // if we find something that isn't a choice, after we've gotten
            //     // to choices, then that's an error
            //     if !reached_choices {
            //         section_description.push_str(&format!("{}\n", line));
            //         continue;
            //     } else {
            //         return Err(ParseError::new(ParseErrorType::ExpectedChoice, line_num));
            //     }
            // };
            // if we've reached this far, we've successfully parsed it as a
            // choice
            reached_choices = true;
            choices.push(choice);
        }

        // TODO: refactor line and line_num

        // if we end without finding any description, that's an error, and
        // idem for choices
        if section_description.is_empty() {
            return Err(ParseError::new(
                ParseErrorType::ExpectedDescription,
                line_num.unwrap(),
                current_line.unwrap().to_owned(),
            ));
        } else if choices.is_empty() {
            return Err(ParseError::new(
                ParseErrorType::ExpectedChoice,
                line_num.unwrap(),
                current_line.unwrap().to_owned(),
            ));
        }

        let section_description = Description::new(&section_description);

        let mut section = Self {
            identifier: section_identifier,
            description: section_description,
            choices,
            line_num: line_num.unwrap(),
        };

        if let "END" | "__RESTART" | "__MENU" = section.identifier.0.as_str() {
            return Err(ParseError::new(
                ParseErrorType::ReservedKeyUsage,
                line_num.unwrap(),
                current_line.unwrap().to_owned(),
            ));
        }

        let len = section.choices.len();
        for choice in &mut section.choices {
            if let "__RESTART" | "__MENU" = choice.goto.0.as_str() {
                return Err(ParseError::new(
                    ParseErrorType::ReservedKeyUsage,
                    line_num.unwrap(),
                    current_line.unwrap().to_owned(),
                ));
            }

            if choice.description.to_string().is_empty() {
                if len != 1 {
                    return Err(ParseError::new(
                        ParseErrorType::ChoiceShorthandNotLone,
                        line_num.unwrap(),
                        current_line.unwrap().to_owned(),
                    ));
                }

                if choice.goto.0 == "END" {
                    section.choices = vec![
                        Choice::parse("Restart from beginning -> __RESTART", line_num.unwrap())
                            .expect("manually verified"),
                        Choice::parse("Return to menu -> __MENU", line_num.unwrap())
                            .expect("manually verified"),
                    ];
                } else {
                    choice.description = Description::new("Continue...");
                }
                break;
            } else if choice.goto.0 == "END" {
                return Err(ParseError::new(
                    ParseErrorType::InvalidEnd,
                    line_num.unwrap(),
                    current_line.unwrap().to_owned(),
                ));
            }
        }

        Ok(section)
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<div id="description">
    <p>
        {}
    </p>
</div>
<div id="choices">"#,
            self.description
        )?;

        for choice in &self.choices {
            write!(
                f,
                r#"<div class="choice" data-fater-goto="{}">
    <span>
        {}
    </span>
</div>"#,
                choice.goto, choice.description
            )?;
        }

        write!(f, "</div>")
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
struct Choice {
    description: Description,
    goto: SectionIdentifier,
}

impl Choice {
    fn parse(str: &str, line_num: usize) -> ParseResult<Self> {
        // we want to parse it as the following: any characters, until we get
        // to an ->, and then a section identifier
        // any requirements on the description? pretty much nothing

        let mut parts: Vec<&str> = str.split("->").collect();

        if (parts.len() < 2) || parts[0].ends_with('\\') {
            return Err(ParseError::new(
                ParseErrorType::MissingArrow,
                line_num,
                str.to_owned(),
            ));
        } else if parts.len() > 2 {
            return Err(ParseError::new(
                ParseErrorType::MultipleArrows,
                line_num,
                str.to_owned(),
            ));
        }

        if parts[0].is_empty() {
            // TODO: should this be how it works?
            // or should we do this later on, when actually validating symbols?
            // probably the latter, since END is special-cased
            // parts[0] = "Continue...";
        }

        let goto = SectionIdentifier::parse((line_num, parts[1]), false)?;
        let description = Description::new(parts[0]);

        Ok(Self { description, goto })
    }
}

// just a newtype, that's all caps, numeric, and underscores
// must have *some* alphabetic. can't be all underscores/numeric
#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct SectionIdentifier(String);

// TODO: figure out solution for these being public
impl SectionIdentifier {
    // def is whether it's a definition (in which case we want a final colon),
    // or not, in which case we don't
    pub fn parse((line_num, mut str): (usize, &str), def: bool) -> ParseResult<Self> {
        let mut found_alphabetic = false;
        let mut colon = false;

        str = str.trim();

        for ch in str.chars() {
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
                return Err(ParseError::new(
                    ParseErrorType::SectionIdentifier(ch),
                    line_num,
                    str.to_owned(),
                ));
            }
        }

        if colon {
            // get rid of the colon
            str = &str[..str.len() - 1];
        }

        // either there's a def and a colon, or there's neither
        if def && !colon {
            Err(ParseError::new(
                ParseErrorType::MissingColon,
                line_num,
                str.to_owned(),
            ))
        } else if !def && colon {
            Err(ParseError::new(
                ParseErrorType::UnexpectedColon,
                line_num,
                str.to_owned(),
            ))
        } else if !found_alphabetic {
            Err(ParseError::new(
                ParseErrorType::MissingAlphabetic,
                line_num,
                str.to_owned(),
            ))
        } else {
            Ok(Self(str.to_owned()))
        }
    }
}

impl Display for SectionIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
struct Description(Vec<Paragraph>);

impl Description {
    fn new(str: &str) -> Self {
        // TODO: all we're doing is: parsing into sections
        // then each section eliminates newlines

        let str: String = str
            .trim()
            .lines()
            .map(|line| format!("{}\n", line.trim()))
            .collect();

        let desc = str
            .split("\n\n")
            .map(|paragraph| Paragraph::new(paragraph))
            .collect();

        Self(desc)
    }
}

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, paragraph) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "\n\n")?;
            }
            write!(f, "{}", paragraph)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
struct Paragraph(String);

impl Paragraph {
    fn new(str: &str) -> Self {
        Self(str.replace('\n', " ").trim().to_owned())
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type ParseResult<T> = Result<T, ParseError>;

// #[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
#[derive(Debug)]
pub struct ParseError {
    error_type: ParseErrorType,
    /// starting from 0, normalized to 1 when rendering
    line_num: usize,
    text: String,
}

impl ParseError {
    fn new(error_type: ParseErrorType, line_num: usize, text: String) -> Self {
        Self {
            error_type,
            line_num,
            text,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type)
    }
}

impl Error for ParseError {}

#[derive(Debug)]
pub enum ParseErrorType {
    SectionIdentifier(char),
    UnexpectedColon,
    MissingColon,
    MissingAlphabetic,
    UnexpectedSeparator,
    MissingArrow,
    MultipleArrows,
    ExpectedChoice,
    ExpectedDescription,
    DuplicateSections(Section, Section),
    DanglingGoto,
    InvalidEnd,
    ChoiceShorthandNotLone,
    ReservedKeyUsage,
}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
        //     ParseError::Foo => write!(f, "FOO"),
        // }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story() {
        let story = Story::parse(
            "\
START:
this is a description

yes -> FIRST
no -> SECOND",
        )
        .unwrap();

        // TODO: other stuff
    }

    #[test]
    fn section() {
        assert_eq!(
            Section::parse(
                "\
MU:
foo
bar

baz

foobar -> ALICE
baz buz -> BOB_MARLEY
---"
                .lines()
                .enumerate(),
            )
            .unwrap(),
            Section {
                identifier: SectionIdentifier("MU".to_owned()),
                description: Description(vec![
                    Paragraph("foo bar".to_owned()),
                    Paragraph("baz".to_owned())
                ]),
                choices: vec![
                    Choice {
                        description: Description(vec![Paragraph("foobar".to_owned())]),
                        goto: SectionIdentifier("ALICE".to_owned())
                    },
                    Choice {
                        description: Description(vec![Paragraph("baz buz".to_owned())]),
                        goto: SectionIdentifier("BOB_MARLEY".to_owned())
                    }
                ]
            }
        )
    }

    #[test]
    fn choice() {
        assert_eq!(
            Choice::parse("foo baz -> BAR", 0).unwrap(),
            Choice {
                description: Description(vec![Paragraph("foo baz".to_owned())]),
                goto: SectionIdentifier("BAR".to_owned())
            }
        );
    }

    #[test]
    fn identifier() {
        assert_eq!(
            SectionIdentifier::parse((0, "FOO"), false).unwrap(),
            SectionIdentifier("FOO".to_owned())
        );
        assert_eq!(
            SectionIdentifier::parse((0, "FOO:"), true).unwrap(),
            SectionIdentifier("FOO".to_owned())
        );
    }

    #[test]
    fn description() {
        assert_eq!(
            Description::new("far bar\nbaz\n\nfoobar\nquz").0,
            vec![
                Paragraph("far bar baz".to_owned()),
                Paragraph("foobar quz".to_owned())
            ]
        );
    }

    #[test]
    fn paragraph() {
        assert_eq!(Paragraph::new("  foo\nbar baz  ").0, "foo bar baz");
    }
}
