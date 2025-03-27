# mail-telegram

A service which reads mails from a GMail account and sends it to the user's telegram account.

## Learnings 

- Product fit or use case was missing. One would still rely on an email app like EdisonMail or Gmail for connecting his external mail and receiving emails.
- PDF conversion of email had too many caveats with many cases to either manually convert or write.
- Used too many external reliances like convex.dev for running the cron jobs which eventually made the project go into cold start (and lose interest in the project as well).
- Maybe use an external library or service for authentication at the beginning. Even supabase works.
- Using only 1 language and 1 backend should work. Make the frontend as simple as possible in an api-kind-of project like this.
- Maybe use an AI coding tool like Cursor in the next project. Would also build it over a single weekend.