use teloxide::prelude::*;
use teloxide::types::Document;

pub enum DocumentSource {
  Message,
  Reply,
}

pub fn get_document<'a>(msg: &'a Message) -> Option<(&'a Document, DocumentSource)> {
  if let Some(doc) = msg.document() {
    return Some((doc, DocumentSource::Message));
  }

  if let Some(reply) = msg.reply_to_message() {
    if let Some(doc) = reply.document() {
      return Some((doc, DocumentSource::Reply));
    }
  }

  None
}
