use crate::constants::WORD_LENGTH;
use crate::letter::{Letter, Word};

/// Feedback for a single letter position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LetterFeedback {
    /// Correct letter in correct position (green)
    Correct,
    /// Correct letter in wrong position (yellow)
    WrongPosition,
    /// Letter not in the word (gray)
    NotInWord,
}

/// Complete feedback for a guess
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuessFeedback {
    word: Word,
    feedback: [LetterFeedback; WORD_LENGTH],
}

impl GuessFeedback {
    /// Evaluate a guess against the secret word.
    /// Uses standard Wordle algorithm:
    /// 1. Mark exact matches (green) first
    /// 2. Mark wrong-position matches (yellow) from remaining letters
    /// 3. Remaining letters are not-in-word (gray)
    #[allow(clippy::needless_range_loop)] // Index used across multiple arrays
    pub fn evaluate(guess: &Word, secret: &Word) -> Self {
        let mut feedback = [LetterFeedback::NotInWord; WORD_LENGTH];
        let mut secret_remaining: [Option<Letter>; WORD_LENGTH] = std::array::from_fn(|i| Some(secret.letter(i)));

        // First pass: mark correct positions (green)
        for i in 0..WORD_LENGTH {
            if guess.letter(i) == secret.letter(i) {
                feedback[i] = LetterFeedback::Correct;
                secret_remaining[i] = None;
            }
        }

        // Second pass: mark wrong positions (yellow)
        for i in 0..WORD_LENGTH {
            if feedback[i] == LetterFeedback::Correct {
                continue;
            }
            let guess_letter = guess.letter(i);
            if let Some(pos) = secret_remaining
                .iter()
                .position(|&l| l == Some(guess_letter))
            {
                feedback[i] = LetterFeedback::WrongPosition;
                secret_remaining[pos] = None;
            }
        }

        Self {
            word: guess.clone(),
            feedback,
        }
    }

    /// Get the guessed word
    pub fn word(&self) -> &Word {
        &self.word
    }

    /// Get feedback for each position
    pub fn feedback(&self) -> &[LetterFeedback; WORD_LENGTH] {
        &self.feedback
    }

    /// Check if this is a winning guess (all Correct)
    pub fn is_win(&self) -> bool {
        self.feedback.iter().all(|&f| f == LetterFeedback::Correct)
    }

    /// Iterate over (Letter, LetterFeedback) pairs
    pub fn iter(&self) -> impl Iterator<Item = (Letter, LetterFeedback)> + '_ {
        self.word.letters().zip(self.feedback.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_correct() {
        let guess = Word::parse("hello").unwrap();
        let secret = Word::parse("hello").unwrap();
        let feedback = GuessFeedback::evaluate(&guess, &secret);

        assert!(feedback.is_win());
        assert_eq!(
            feedback.feedback(),
            &[LetterFeedback::Correct; WORD_LENGTH]
        );
    }

    #[test]
    fn test_all_wrong() {
        let guess = Word::parse("xxxxx").unwrap();
        let secret = Word::parse("hello").unwrap();
        let feedback = GuessFeedback::evaluate(&guess, &secret);

        assert!(!feedback.is_win());
        assert_eq!(
            feedback.feedback(),
            &[LetterFeedback::NotInWord; WORD_LENGTH]
        );
    }

    #[test]
    fn test_wrong_position() {
        let guess = Word::parse("olleh").unwrap();
        let secret = Word::parse("hello").unwrap();
        let feedback = GuessFeedback::evaluate(&guess, &secret);

        assert!(!feedback.is_win());
        // o is in wrong position, l is correct, l is correct, e is wrong position, h is wrong position
        assert_eq!(
            feedback.feedback(),
            &[
                LetterFeedback::WrongPosition,
                LetterFeedback::WrongPosition,
                LetterFeedback::Correct,
                LetterFeedback::WrongPosition,
                LetterFeedback::WrongPosition,
            ]
        );
    }

    #[test]
    fn test_duplicate_letters() {
        // Guess has duplicate 'l', secret has two 'l's
        let guess = Word::parse("llama").unwrap();
        let secret = Word::parse("hello").unwrap();
        let feedback = GuessFeedback::evaluate(&guess, &secret);

        // First 'l' is wrong position (matches one 'l' in hello)
        // Second 'l' is wrong position (matches the other 'l' in hello)
        // 'a' is not in word, 'm' is not in word, 'a' is not in word
        assert_eq!(
            feedback.feedback(),
            &[
                LetterFeedback::WrongPosition,
                LetterFeedback::WrongPosition,
                LetterFeedback::NotInWord,
                LetterFeedback::NotInWord,
                LetterFeedback::NotInWord,
            ]
        );
    }

    #[test]
    fn test_duplicate_letters_one_correct() {
        // Guess: "hello", Secret: "hella"
        // Both have 'l' at positions 2 and 3
        let guess = Word::parse("hello").unwrap();
        let secret = Word::parse("hella").unwrap();
        let feedback = GuessFeedback::evaluate(&guess, &secret);

        assert_eq!(
            feedback.feedback(),
            &[
                LetterFeedback::Correct,     // h
                LetterFeedback::Correct,     // e
                LetterFeedback::Correct,     // l
                LetterFeedback::Correct,     // l
                LetterFeedback::NotInWord,   // o (not in hella)
            ]
        );
    }

    #[test]
    fn test_extra_duplicate_in_guess() {
        // Guess: "geese", Secret: "eerie"
        // 'e' appears 3 times in guess, 3 times in secret
        let guess = Word::parse("geese").unwrap();
        let secret = Word::parse("eerie").unwrap();
        let feedback = GuessFeedback::evaluate(&guess, &secret);

        // g: not in word
        // e (pos 1): correct (secret also has 'e' at pos 1)
        // e (pos 2): wrong position (secret has 'e' at 0 and 4, 'r' at 2)
        // s: not in word
        // e (pos 4): correct (secret has 'e' at pos 4)
        assert_eq!(
            feedback.feedback(),
            &[
                LetterFeedback::NotInWord,
                LetterFeedback::Correct,
                LetterFeedback::WrongPosition,
                LetterFeedback::NotInWord,
                LetterFeedback::Correct,
            ]
        );
    }
}
