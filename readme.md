
# Pomodoro Timer

A command line pomodoro timer.

## Install

### Option 1
- clone repo and run cargo install --path .

### Option 2 
- prerequisite: archlinux/manjaro and yay
```yay -S polpettone-pomodoro-timer```


## Usage

### Tipps 
Set an alias in your shell.

For example: 

```
alias ppt="polpettone-pomodoro-timer"
```

### Start a session
```
ppt start -d 'hacken'
```
Initial run of the command will create a config file ~/.config/polpettone-pomodoro-timer/config.toml

In the config.toml is the path setting for the session files.
Each sessions is saved in one yaml file.

this is the default: 
```
pomodoro_session_dir = "~/polpettone-pomodoro-sessions"
```

U need to create this directory manually or run 
```
ppt --init-session-dir
```

A status file with the status of the current or last pomodoro session is in 
this directory also. ###You can use this to integrate this to your polybar or another kind of status bar.

### Watch a running session
```
ppt watch 
```
This will watch and update the timer of the last running session.
U need to run this to update the status file.


### Get your sessions 
```
ppt find-session-from-today
```
This will show you the pomodoro sessions from today.
There also commands to get your session for a specific time range.

#### Find session in a time range
```
ppt find-sessions-in-range "2025-01-01 00:00:00" "2026-01-31 23:59:59" 
```

#### Find session in a time range with search query 
```
ppt find-sessions-in-range "2025-01-01 00:00:00" "2026-01-31 23:59:59"  -s pomo
```

```
+-------------+----------+---------------------+
| Description | Duration | Start Time          |
+==============================================+
| pomo timer  | "25:0"   | 2025-05-31 14:05:26 |
|-------------+----------+---------------------|
| pomo timer  | "25:0"   | 2025-05-31 13:30:57 |
+-------------+----------+---------------------+
```

#### ASCII Table output
Use the -e Flag to change the output format to an ASCII Table
```
ppt find-sessions-in-range "2025-01-01 00:00:00" "2026-01-31 23:59:59"  -s pomo -e 
```

```
|   No   |           Start       |   Dauer   |     Beschreibung   |
|--------|-----------------------|-----------|--------------------|
|      1 | 2025-05-31 13:30:57   | 25:00     | pomo timer         |
|      2 | 2025-05-31 14:05:26   | 25:00     | pomo timer         |
| Total  |            --         | 00:50     |      --------      |
 
```

### Help 
For more commands run 

```
polpettone-pomodoro-timer help
```


