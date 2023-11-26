# cli-timer

A simple terminal application that plays a sound after the specified duration has elapsed.

## Usage

Timer for 5 minutes:
```bash
timer 5
```

Or a more complex duration:
```
timer 1:30:10
```

A default sound is included in the binary, however you may specify your custom alarm sound by specifying the path to it:
```bash
timer 5 --alarm-path "/path/to/your/sound.mp3"
```