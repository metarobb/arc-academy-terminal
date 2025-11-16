# Interactive Lessons Guide

## How to Use Lessons

### Starting a Lesson

1. **Launch Arc Academy Terminal**:
   ```bash
   cargo run --release
   ```

2. **Activate Lesson Mode**:
   - Press `F2` to toggle lesson mode ON
   - You'll see "📖 LESSON MODE" in the header
   - The right panel changes from "Learning" to show the lesson

3. **The lesson "Navigation Basics" auto-loads on first activation**

### Lesson Controls

| Key | Action |
|-----|--------|
| `F2` | Toggle lesson mode ON/OFF (primary) |
| `Ctrl+E` | Alternative toggle (if your terminal supports it) |
| `Tab` | Switch between panels |
| `Enter` | Submit your command answer |
| `?` | Show help overlay |

### How Lessons Work

#### Command Exercises
When you see a step like:
```
Step 1: Understanding Your Current Location
Type the command to print your current working directory.

💡 Hint: The command is 'pwd' (print working directory)
```

**What to do:**
1. Type the command in the Shell panel (top right)
2. Press `Enter`
3. The system validates your answer:
   - ✅ **Correct**: Automatically moves to next step
   - ❌ **Incorrect**: Shows hint and lets you try again

#### Multiple Choice Questions
When you see a quiz:
```
Step 6: Quiz: What does 'cd ..' do?

❓ What does the command 'cd ..' do?

  0. Goes to the home directory
  1. Goes up one directory level (to the parent)
  2. Lists files in the current directory
  3. Creates a new directory

▶ Enter the number of your answer
```

**What to do:**
1. Type the number (0, 1, 2, or 3)
2. Press `Enter`
3. Correct answer shows explanation and moves forward
4. Wrong answer lets you try again

#### Information Steps
Some steps just display information:
```
Step 7: Pro Tip: cd -

Pro tip: 'cd -' is a super useful command! It takes you back to your
previous directory. Try it:

  cd /tmp
  cd ~
  cd -    # Takes you back to /tmp!

▶ Press Enter to continue
```

**What to do:**
1. Read the information
2. Press `Enter` to continue

### Progress Tracking

The lesson panel header shows:
```
📖 Navigation Basics | Step 3/8 | 25%
```

- **Current step** / Total steps
- **Completion percentage**
- Steps you've completed are tracked

### Available Lessons

#### 1. Navigation Basics (10 min, Beginner 🌱)
Learn essential navigation commands:
- `pwd` - Print working directory
- `ls` - List files (with -l, -a, -h flags)
- `cd` - Change directory
- `cd ~` - Go home
- `cd ..` - Go up one level
- `cd -` - Toggle to previous directory

**8 Steps**: Command exercises, quizzes, and pro tips

#### 2. File Management (15 min, Beginner 🌱)
Learn to manage files safely:
- `mkdir` - Create directories
- `touch` - Create files
- `cp` - Copy files
- `mv` - Move/rename files
- `rm -i` - Delete safely (with confirmation)

**7 Steps**: Includes safety quiz and strong warnings about rm -rf

### Lesson Completion

When you finish all steps:
```
🎉 Congratulations! You've completed this lesson!

Press F2 to exit lesson mode.
```

Press `F2` to:
- Return to normal shell mode
- Try another lesson (future feature)
- Continue practicing what you learned

## Tips

### Best Practices
1. **Read each step carefully** - Instructions tell you exactly what to do
2. **Use the hints** - They're there to help you learn
3. **Don't rush** - Take time to understand each concept
4. **Practice outside lessons** - Try commands in normal shell mode too

### Common Issues

**Q: F2 doesn't work / I prefer keyboard shortcuts**
- Try `Ctrl+E` as an alternative (works in some terminals)
- Some terminals may intercept F2 - check your terminal settings
- If both fail, this is usually a terminal emulator configuration issue

**Q: I pressed F2/Ctrl+E but don't see a lesson**
- The lesson should auto-load. Try pressing `F2` twice to toggle off and on again
- Check the Output panel for activation message
- Make sure you're not in the onboarding wizard or settings panel

**Q: My command was correct but it says it's wrong**
- Check spacing and spelling exactly
- Some steps accept multiple variations (e.g., "ls -lh" or "ls -hl")
- Read the hint for the expected format

**Q: I want to skip a step**
- Currently not supported - lessons are designed to be completed in order
- Each step builds on previous knowledge

**Q: How do I go back to a previous step?**
- Currently not supported - lessons move forward only
- You can restart by toggling lesson mode off and on (Ctrl+E twice)

## Coming Soon

### Planned Lessons
- **Text Processing** - grep, sed, awk
- **Git Fundamentals** - init, add, commit, push, pull
- **System Administration** - systemctl, journalctl, users
- **Shell Scripting Intro** - variables, loops, conditionals

### Planned Features
- Lesson selection menu
- Progress persistence (save your place)
- Achievement badges
- Step navigation (next/previous)
- Lesson restart option
- Custom user lessons

## Development

Want to create your own lesson? See the lesson framework in:
- `crates/arct-core/src/lesson.rs` - Core data structures
- `crates/arct-tui/src/panels/lesson.rs` - UI rendering
- Look at `create_navigation_basics_lesson()` for an example

---

**Happy Learning!** 🚀

For issues or questions, visit: https://github.com/metarobb/arc-academy-terminal
