
# Pomodoro Timer

A command line pomodoro timer.

## Install

### Option 1
- clone repo and run cargo install --path .

### Option 2 
- prerequisite: archlinux/manjaro and yay
```yay -S polpettone-pomodoro-timer```


## Usage

### Start a session
```
polpettone-pomodoro-timer start -d 'hacken'
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
polpettone-pomodoro-timer --init-session-dir
```

A status file with the status of the current or last pomodoro session is in 
this directory also. 
You can use this to integrate this to your polybar or another kind of status bar.

### Watch a running session
```
polpettone-pomodoro-timer watch 
```
This will watch and update the timer of the last running session.
U need to run this to update the status file.


### Get your sessions 
```
polpettone-pomodoro-timer find-session-from-today
```
This will show you the pomodoro sessions from today.
There also commands to get your session for a specific time range.


### Help 
For more commands run 

```
polpettone-pomodoro-timer help
```
