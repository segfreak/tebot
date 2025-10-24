use teloxide::prelude::*;
use teloxide::types::*;

use bitflags::bitflags;

bitflags! {
  pub struct GetParameters: u8 {
    const MESSAGE = 0b00000001;
    const REPLY   = 0b00000010;
  }
}

pub enum DocumentSource {
  Message,
  Reply,
}

pub fn get_document<'a>(
  msg: &'a Message,
  params: GetParameters,
) -> Option<(&'a Document, DocumentSource)> {
  if params.contains(GetParameters::MESSAGE) {
    if let Some(doc) = msg.document() {
      return Some((doc, DocumentSource::Message));
    }
  }

  if params.contains(GetParameters::REPLY) {
    if let Some(reply) = msg.reply_to_message() {
      if let Some(doc) = reply.document() {
        return Some((doc, DocumentSource::Reply));
      }
    }
  }

  None
}

pub fn get_audio<'a>(
  msg: &'a Message,
  params: GetParameters,
) -> Option<(&'a Audio, DocumentSource)> {
  if params.contains(GetParameters::MESSAGE) {
    if let Some(audio) = msg.audio() {
      return Some((audio, DocumentSource::Message));
    }
  }

  if params.contains(GetParameters::REPLY) {
    if let Some(reply) = msg.reply_to_message() {
      if let Some(audio) = reply.audio() {
        return Some((audio, DocumentSource::Reply));
      }
    }
  }

  None
}

pub fn get_video<'a>(
  msg: &'a Message,
  params: GetParameters,
) -> Option<(&'a Video, DocumentSource)> {
  if params.contains(GetParameters::MESSAGE) {
    if let Some(video) = msg.video() {
      return Some((video, DocumentSource::Message));
    }
  }

  if params.contains(GetParameters::REPLY) {
    if let Some(reply) = msg.reply_to_message() {
      if let Some(video) = reply.video() {
        return Some((video, DocumentSource::Reply));
      }
    }
  }

  None
}

fn get_largest_photo<'a>(m: &'a Message) -> Option<&'a PhotoSize> {
  m.photo()
    .and_then(|photos| photos.iter().max_by_key(|p| p.width * p.height))
}

pub fn get_picture<'a>(
  msg: &'a Message,
  params: GetParameters,
) -> Option<(&'a PhotoSize, DocumentSource)> {
  if params.contains(GetParameters::MESSAGE) {
    if let Some(photo) = get_largest_photo(msg) {
      return Some((photo, DocumentSource::Message));
    }
  }

  if params.contains(GetParameters::REPLY) {
    if let Some(reply) = msg.reply_to_message() {
      if let Some(photo) = get_largest_photo(reply) {
        return Some((photo, DocumentSource::Reply));
      }
    }
  }

  None
}
