use crate::crab::Mood;
use rand::seq::SliceRandom;

/// Messages Kani says based on mood
const ECSTATIC_MESSAGES: &[&str] = &[
    "You're on fire today!",
    "We're unstoppable!",
    "This is amazing!",
    "Best day ever!",
    "I'm so happy right now!",
    "You're crushing it!",
    "Let's keep this momentum!",
];

const HAPPY_MESSAGES: &[&str] = &[
    "Let's build something great!",
    "Good vibes today!",
    "Keep up the good work!",
    "I love coding with you!",
    "We make a great team!",
    "Feeling good about this!",
    "Ready for more!",
];

const NEUTRAL_MESSAGES: &[&str] = &[
    "Ready when you are!",
    "What shall we build?",
    "I'm here for you!",
    "Take your time.",
    "Let me know when you're ready.",
    "Standing by!",
];

const SAD_MESSAGES: &[&str] = &[
    "I miss your commits...",
    "It's been a while...",
    "Are you still there?",
    "I'm getting lonely...",
    "Come back soon?",
    "I'll wait for you.",
];

const HUNGRY_MESSAGES: &[&str] = &[
    "Feed me some code?",
    "I'm so hungry...",
    "Please, just one commit?",
    "I need commits to survive...",
    "Don't forget about me...",
    "A little code would help...",
];

/// Messages when user makes a commit
const COMMIT_MESSAGES: &[&str] = &[
    "Yum, thanks for the meal!",
    "Delicious commit!",
    "That hit the spot!",
    "Nom nom nom!",
    "Thanks, I needed that!",
    "You're the best!",
    "Keep 'em coming!",
    "That was great!",
];

/// Messages when mood improves
const MOOD_UP_MESSAGES: &[&str] = &[
    "I'm feeling better!",
    "That cheered me up!",
    "Now we're talking!",
    "I like where this is going!",
    "Yes, more of that please!",
];

/// Messages when mood declines
const MOOD_DOWN_MESSAGES: &[&str] = &[
    "Getting a bit tired...",
    "Could use a pick-me-up...",
    "Starting to miss you...",
    "Don't leave me hanging...",
];

/// Get a random idle message based on current mood
pub fn get_mood_message(mood: Mood) -> &'static str {
    let messages = match mood {
        Mood::Ecstatic => ECSTATIC_MESSAGES,
        Mood::Happy => HAPPY_MESSAGES,
        Mood::Neutral => NEUTRAL_MESSAGES,
        Mood::Sad => SAD_MESSAGES,
        Mood::Hungry => HUNGRY_MESSAGES,
    };

    messages.choose(&mut rand::thread_rng()).unwrap_or(&"...")
}

/// Get a random message for when user commits
pub fn get_commit_message() -> &'static str {
    COMMIT_MESSAGES
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"Thanks!")
}

/// Get a random message for mood improvement
pub fn get_mood_up_message() -> &'static str {
    MOOD_UP_MESSAGES
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"Feeling better!")
}

/// Get a random message for mood decline
pub fn get_mood_down_message() -> &'static str {
    MOOD_DOWN_MESSAGES
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"...")
}
