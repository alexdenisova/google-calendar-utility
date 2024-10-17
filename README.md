# google-calendar-utility

Utility to sign up for classes as well as add them to Google Calendar.

## Commands

`google-calendar-utility sign-up` - sign up to classes listed in config.
`google-calendar-utility sync` - sync those classes with Google Calendar Events.

## Sign Up Config

```yaml
timezone: Europe/Moscow
offsetWeeks: 1 # sign up to classes that will be in 1 week
holiYoga:
  - name: "Хатха йога (All Level)"
    weekday: Mon
    startTime: 19:00
plastilin:
  - name: Гибкая спина
    weekday: Sat
    startTime: 13:00
```

## Environment Variables

* `GCU__DEBUG` (optional, `=false`) - set logging level to debug.
* `GCU__SIGN_UP_CONFIG` (mandatory for `sign-up`) - path to sign up config.
* `GCU__GOOGLE_EMAIL` (mandatory for `sync`) - email address of service account.
* `GCU__GOOGLE_KEY_ID` (mandatory for `sync`) - service account private key id.
* `GCU__GOOGLE_PRIVATE_KEY` (mandatory for `sync`) - path to service account private key.
* `GCU__GOOGLE_CALENDAR_ID` (mandatory for `sync`) - Google calendar id (usually your email address).
* `GCU__HOLI_USERNAME` (mandatory for Holi Yoga) - Holi Yoga username (phone number like 79123456789).
* `GCU__HOLI_PASSWORD` (mandatory for Holi Yoga) - Holi Yoga password.
* `GCU__HOLI_API_KEY` (optional, `="63b92ce0-3a63-4de5-8ee0-2756b62a0190"`) - Holi Yoga api key (api_key in request forms).
* `GCU__HOLI_CLUB_ID` (optional, `="3dc77e1c-434c-11ea-bbc1-0050568bac14"`) - Holi Yoga club id.
* `GCU__PLASTILIN_TOKEN` (mandatory for Plastilin) - token for authorization.
* `GCU__PLASTILIN_CLUB_ID` (optional, `="1820"`) - Plastilin club id.
