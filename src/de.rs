use std::ops::{Neg, AddAssign, MulAssign};

use serde::de::{self, Deserialize, DeserializeSeed, Visitor, EnumAccess, VariantAccess, IntoDeserializer};

use error::{Error, Result};
use std::str::FromStr;

pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input
        }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
    where T: Deserialize<'a>
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        if self.input.starts_with("\"1\"") {
            self.input = &self.input["\"1\"".len()..];
            Ok(true)
        } else if self.input.starts_with("\"0\"") {
            self.input = &self.input["\"0\"".len()..];
            Ok(false)
        } else {
            Err(Error::ExpectedBoolean)
        }
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
        where T: AddAssign<T> + MulAssign<T> + From<u8>
    {
        match self.next_char()? {
            '"' => (),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        }
        let mut int = match self.next_char()? {
            ch @ '0' ... '9' => T::from(ch as u8 - b'0'),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        loop {
            match self.input.chars().next() {
                Some(ch @ '0' ... '9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch as u8 - b'0');
                }
                Some('"') => {
                    self.input = &self.input[1..];
                    return Ok(int);
                },
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }
        }
    }

    fn parse_signed<T>(&mut self) -> Result<T>
        where T: Neg<Output=T> + AddAssign<T> + MulAssign<T> + FromStr
    {
        match self.peek_char()? {
            '"' => {
                self.next_char()?;
                match self.input.find('"') {
                    Some(len) => {
                        let s = &self.input[..len];
                        self.input = &self.input[len + 1..];
                        s.parse::<T>().ok().ok_or(Error::ExpectedInteger)
                    },
                    None => Err(Error::Eof),
                }
            },
            _ => Err(Error::ExpectedString),
        }
    }

    // Parse an escaped String
    fn parse_string(&mut self) -> Result<&'de str> {
        match self.peek_char()? {
            '"' => {
                self.next_char()?;
                match self.input.find('"') {
                    Some(len) => {
                        let s = &self.input[..len];
                        self.input = &self.input[len + 1..];
                        Ok(s)
                    },
                    None => Err(Error::Eof),
                }
            },
            _ => match self.input.find(' ') {
                Some(len) => {
                    let s = &self.input[..len];
                    self.input = &self.input[len + 1..];
                    Ok(s)
                },
                None => {
                    let s = self.input;
                    self.input = "";
                    Ok(s)
                }
            },
            ' ' => Err(Error::ExpectedString),
        }
    }

    // Parse an unescaped string
    fn parse_command(&mut self) -> Result<&'de str> {
        match self.input
            .find(' ')
            .or(Some(self.input.len())) {
            Some(len) => {
                let s = &self.input[..len];
                self.input = &self.input[len..];
                Ok(s)
            }
            None => Err(Error::Eof),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        let value = match self.input.find(' ') {
            None => visitor.visit_enum(self.parse_command()?.into_deserializer())?,
            Some(_) => visitor.visit_enum(Enum::new(self))?
        };
        match self.next_char() {
            Err(Error::Eof) => Ok(value),
            Ok('\n') => Ok(value),
            _ => Err(Error::ExpectedCommandNewline)
        }
    }

    fn deserialize_identifier<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_borrowed_str(self.parse_command()?)
    }

    fn deserialize_ignored_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        self.deserialize_any(visitor)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum {
            de
        }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: DeserializeSeed<'de>
    {
        let val = seed.deserialize(&mut *self.de)?;
        let next = self.de.next_char()?;
        if next == ' ' {
            Ok((val, self))
        } else {
            Err(Error::ExpectedCommandSpace)
        }
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed<'de>
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
        where V: Visitor<'de>
    {
        unimplemented!();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_command() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum Commands {
        #[serde(rename = "status")]
        Status
    }

    let j = "status";
    let expected = Commands::Status;
    assert_eq!(expected, from_str(j).unwrap());
}

//#[test]
//fn test_int_command() {
//    #[derive(Deserialize, PartialEq, Debug)]
//    enum Commands {
//        #[serde(rename = "play")]
//        Play(u64)
//    }
//
//    let n = "play \"4\"";
//    let expected = Commands::Play(4);
//    assert_eq!(expected, from_str(n).unwrap());
//}

#[test]
fn test_bool_command() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum Commands {
        #[serde(rename = "pause")]
        Pause(bool)
    }

    let o = "pause \"1\"";
    let expected = Commands::Pause(true);
    assert_eq!(expected, from_str(o).unwrap());

    let p = "pause \"0\"";
    let expected = Commands::Pause(false);
    assert_eq!(expected, from_str(p).unwrap());
}

#[test]
fn test_string_command() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum Commands {
        #[serde(rename = "listplaylist")]
        ListPlaylist(String)
    }

    let q = "listplaylist \"\"";
    let expected = Commands::ListPlaylist(String::new());
    assert_eq!(expected, from_str(q).unwrap());

    let r = "listplaylist \"test\"";
    let expected = Commands::ListPlaylist(String::from("test"));
    assert_eq!(expected, from_str(r).unwrap());

    let s = "listplaylist tet";
    let expected = Commands::ListPlaylist(String::from("tet"));
    assert_eq!(expected, from_str(s).unwrap());
}