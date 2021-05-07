use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GoogleCalendarAccessRole {
    Owner,
    Writer,
    Reader,
    FreeBusyReader,
}

// {\n
//     \"kind\": \"calendar#calendarListEntry\",\n
//     \"etag\": \"\\\"1613088395199000\\\"\",\n
//     \"id\": \"admin@nettu.no\",\n
//     \"summary\": \"admin@nettu.no\",\n
//     \"timeZone\": \"UTC\",\n
//     \"colorId\": \"14\",\n
//     \"backgroundColor\": \"#9fe1e7\",\n
//     \"foregroundColor\": \"#000000\",\n
//     \"selected\": true,\n
//     \"accessRole\": \"owner\",\n
//     \"defaultReminders\": [\n
//     {\n
//         \"method\": \"popup\",\n
//         \"minutes\": 10\n
//     }\n
//     ],\n
//     \"notificationSettings\": {\n
//         \"notifications\": [\n
//         {\n
//             \"type\": \"eventCreation\",\n
//             \"method\": \"email\"\n
//         },\n
//         {\n
//             \"type\": \"eventChange\",\n
//             \"method\": \"email\"\n
//         },\n
//         {\n
//             \"type\": \"eventCancellation\",\n
//             \"method\": \"email\"\n
//         },\n
//         {\n
//             \"type\": \"eventResponse\",\n
//             \"method\": \"email\"\n
//         }\n
//         ]\n
//     },\n
//     \"primary\": true,\n
//     \"conferenceProperties\":
//         {\n
//             \"allowedConferenceSolutionTypes\":
//             [\n     \"hangoutsMeet\"\n    ]\n
//         }\n
//     }\n
//     ]\n
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarListEntry {
    pub id: String,
    pub access_role: GoogleCalendarAccessRole,
    pub summary: String,
    pub summary_override: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub time_zone: Option<String>,
    pub color_id: Option<String>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub hidden: Option<bool>,
    pub selected: Option<bool>,
    pub primary: Option<bool>,
    pub deleted: Option<bool>,
}
