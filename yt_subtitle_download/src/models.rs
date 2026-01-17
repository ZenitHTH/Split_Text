use serde::Deserialize;

// โครงสร้างหลักของข้อมูลที่ได้จาก ytInitialPlayerResponse
#[derive(Deserialize, Debug)]
pub struct PlayerResponse {
    pub captions: Option<Captions>,
    pub video_details: Option<VideoDetails>,
}

#[derive(Deserialize, Debug)]
pub struct Captions {
    #[serde(rename = "playerCaptionsTracklistRenderer")]
    pub renderer: PlayerCaptionsRenderer,
}

#[derive(Deserialize, Debug)]
pub struct PlayerCaptionsRenderer {
    #[serde(rename = "captionTracks")]
    pub tracks: Vec<CaptionTrack>,
}

#[derive(Deserialize, Debug)]
pub struct CaptionTrack {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub name: Name,
    #[serde(rename = "languageCode")]
    pub lang_code: String,
}

#[derive(Deserialize, Debug)]
pub struct Name {
    #[serde(rename = "simpleText")]
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct VideoDetails {
    pub title: String,
    #[serde(rename = "videoId")]
    pub video_id: String,
}

// โครงสร้างสำหรับข้อมูล Transcript (json3)
#[derive(Deserialize, Debug)]
pub struct TranscriptResponse {
    pub events: Vec<TranscriptEvent>,
}

#[derive(Deserialize, Debug)]
pub struct TranscriptEvent {
    #[serde(rename = "tStartMs")]
    pub t_start_ms: Option<u64>,
    #[serde(rename = "dDurationMs")]
    pub d_duration_ms: Option<u64>,
    pub segs: Option<Vec<Segment>>,
}

#[derive(Deserialize, Debug)]
pub struct Segment {
    pub utf8: String,
}