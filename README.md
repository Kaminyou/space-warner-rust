# space-warner-rust
Send a message to your Slack whenever your space is becoming full!

## Setup
1. Create `.env`
```
API_ENDPOINT=  # YOUR SLACK API ENDPOINT
FILE_SYSTEMS=  # separated by ,
THRESHOLD=  # e.g., 0.70 -> send a message when space usage >= 70%
TRIGGER_INTERVAL=60  # second # if there is no warning triggered previously -> interval to check the system
WARNING_INTERVAL=3600  # second # once a warning is triggered, trigger again after WARNING_INTERVAL
```
2. 
```
$ docker-compose up --build -d
```