use teloxide::prelude::*;
use teloxide::types::*;

use bitflags::bitflags;

bitflags! {
  pub struct GetParameters: u8 {
    const MESSAGE = 0b00000001;
    const REPLY   = 0b00000010;
  }
}

/// Represents the origin of a found media element.
///
/// Used to identify whether a file (document, image, etc.)
/// was found in the current message or within a reply.
pub enum DocumentSource {
  /// The content originates from the main message.
  Message,
  /// The content originates from a replied-to message.
  Reply,
}

/// Attempts to extract a [`Document`] from the given message.
///
/// The search behavior is controlled by [`GetParameters`],
/// allowing lookup in the message itself or in its reply.  
///
/// Returns both the document reference and its source if found.
/// Returns `None` when no matching document exists.
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

/// Attempts to extract an [`Audio`] file from the given message.
///
/// The search can include both the main message and the reply message,
/// depending on [`GetParameters`].  
///
/// Returns a reference to the audio object and its origin if found,
/// or `None` if no valid audio file exists.
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

/// Attempts to extract a [`Video`] from the given message.
///
/// This function checks for video attachments in the current
/// message and optionally in the replied-to message.  
///
/// Returns a tuple containing the video reference and its source.
/// Returns `None` when no video is found.
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

/// Returns the largest available [`PhotoSize`] from a message.
///
/// Used internally to select the best-quality version of an image.
/// Returns `None` if the message contains no photos.
fn get_largest_photo<'a>(m: &'a Message) -> Option<&'a PhotoSize> {
  m.photo()
    .and_then(|photos| photos.iter().max_by_key(|p| p.width * p.height))
}

/// Attempts to extract a photo from the given message.
///
/// Checks both the message and its reply for images, selecting
/// the largest available version of the photo.  
///
/// Returns the image reference and its source if found,
/// or `None` if no image is available.
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
