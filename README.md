# Connect Console

Wall chart for an organization. Same Postgres as the plane. Sign in as a **person**, who is also a plane **client** — the same agent the laptop uses.

There is no stock CMS in this stack, so the console *is* a small one: collections of records, each with fields and actions gated by role. The UI only renders what `/api/orgs/{name}/wall` returns. The wall for acme is `http://127.0.0.1:3040/acme`.

- **owner** — first person in the org. Promote admins, assign machines.
- **admin** — people, machines, ACL, pending logins.
- **member** — only their own machines.

Does **not** dial the plane or carry App or Session. That is still `connect-client`.

Screencast (product lab): `./scripts/record-demo.sh` → `docs/demo.mp4`.

```
connect-console --config /etc/connect/connect.toml
# http://127.0.0.1:3040  → create account or sign in
```

Laptop still dials the **plane**. Login still goes through **connect-gateway** if you use device/OIDC/key.
