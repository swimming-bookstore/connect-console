import { expect, test, type Page } from "@playwright/test";

const EMAIL = "alice@acme.test";
const PASS = "lab-alice-8";

async function signIn(page: Page) {
  await page.goto("/");
  await expect(page.getByTestId("gate")).toBeVisible();
  await page.getByTestId("email").fill(EMAIL);
  await page.getByTestId("password").fill(PASS);
  await page.getByTestId("submit").click();
  await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
}

async function openOrg(page: Page) {
  await signIn(page);
  await page.getByTestId("home-org").click();
  await expect(page).toHaveURL(/\/org\/acme$/);
  await expect(page.getByTestId("page-org")).toBeVisible();
  await page.getByTestId("org-boxes").click();
  await expect(page).toHaveURL(/\/org\/acme\/boxes$/);
}

test.describe.serial("console", () => {
  test("create account is email and password only", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("Create an account, then add boxes")).toHaveCount(0);
    await expect(page.getByText("No account yet?")).toHaveCount(0);
    await expect(page.getByText("Password at least 8 characters")).toHaveCount(0);
    await expect(page.getByTestId("submit")).toHaveClass(/btn-ink/);
    await expect(page.getByTestId("submit")).toHaveText("Sign in");
    const bg = await page.locator("html").evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(bg).toBe("rgb(255, 255, 255)");
    await page.getByTestId("tab-create").click();
    await expect(page.getByTestId("email")).toBeVisible();
    await expect(page.getByTestId("password")).toBeVisible();
    await expect(page.getByTestId("person-name")).toHaveCount(0);
    await expect(page.getByTestId("org-name")).toHaveCount(0);
    await expect(page.getByTestId("submit")).toHaveText("Create account");
    await expect(page.getByTestId("submit")).toHaveClass(/btn-ink/);
    await page.getByTestId("tab-signin").click();
    await expect(page.getByTestId("submit")).toHaveText("Sign in");
    await expect(page.getByText("Click a row to fill")).toHaveCount(0);
    await expect(page.getByTestId("lab-logins")).toBeVisible();
    const submitBox = await page.getByTestId("submit").boundingBox();
    const labBox = await page.getByTestId("lab-logins").boundingBox();
    expect(submitBox && labBox && labBox.y > submitBox.y).toBeTruthy();
    await expect(page.getByTestId("lab-logins")).toContainText("alice@acme.test");
    await expect(page.getByTestId("lab-logins")).toContainText("bob@other-co.test");
    await expect(page.getByTestId("lab-logins")).toContainText("cara@home.test");
  });

  test("gate keeps space between tabs and email", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("gate")).toBeVisible();
    const gap = await page.evaluate(() => {
      const tabs = document.querySelector(".gate-tabs");
      const email = document.querySelector('[data-testid="email"]');
      if (!tabs || !email) return 0;
      const a = tabs.getBoundingClientRect();
      const b = email.getBoundingClientRect();
      return b.top - a.bottom;
    });
    expect(gap).toBeGreaterThanOrEqual(16);
    expect(gap).toBeLessThan(48);
  });

  test("cara lands on home with personal only", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByTestId("home-personal")).toBeVisible();
    await expect(page.getByTestId("home-org")).toHaveCount(0);
    await expect(page.getByTestId("home-create-org")).toBeVisible();
    await page.getByTestId("home-personal").click();
    await expect(page).toHaveURL(/\/personal$/);
    await expect(page.getByTestId("page-org")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Personal");
    await expect(page.getByTestId("org-boxes")).toBeVisible();
    await page.getByTestId("org-boxes").click();
    await expect(page).toHaveURL(/\/personal\/boxes$/);
    await expect(page.getByTestId("org-title")).toHaveText("Boxes");
    await expect(page.getByTestId("crumb-home")).toHaveText("Home");
    await expect(page.getByTestId("crumb-space")).toHaveText("Personal");
  });

  test("alice lands on home and can open personal or org", async ({ page }) => {
    await signIn(page);
    await expect(page.getByTestId("home-personal")).toBeVisible();
    await expect(page.getByTestId("home-org")).toBeVisible();
    await expect(page.getByTestId("home-org")).toContainText("acme");
    await expect(page.getByTestId("page-home").locator(".dir-row")).toHaveCount(2);
    await expect(page.getByTestId("page-home")).not.toContainText("Subpages");
    await expect(page.getByTestId("page-home")).not.toContainText("Page");
    await expect(page.getByTestId("page-home")).not.toContainText("Count");
    await expect(page.getByTestId("who")).toHaveText("alice@acme.test");
    const gap = await page.evaluate(() => {
      const who = document.querySelector("[data-testid='who']");
      const theme = document.querySelector("[data-testid='theme']");
      if (!who || !theme) return 0;
      return theme.getBoundingClientRect().left - who.getBoundingClientRect().right;
    });
    expect(gap).toBeGreaterThanOrEqual(8);
    expect(gap).toBeLessThan(24);
    const count = page.getByTestId("home-org").locator(".count");
    await expect(count).toBeVisible();
    await expect(count).not.toHaveText("");
    await expect(page.getByTestId("home-create-org")).toHaveCount(0);
    await page.getByTestId("home-personal").click();
    await expect(page).toHaveURL(/\/personal$/);
    await expect(page.getByTestId("page-org")).toBeVisible();
    await page.getByTestId("org-boxes").click();
    await expect(page).toHaveURL(/\/personal\/boxes$/);
    await expect(page.getByTestId("org-title")).toHaveText("Boxes");
    await expect(page.getByTestId("crumb-home")).toHaveText("Home");
    await expect(page.getByTestId("crumb-space")).toHaveText("Personal");
    await page.getByTestId("crumb-space").click();
    await expect(page).toHaveURL(/\/personal$/);
    await expect(page.getByTestId("page-org")).toBeVisible();
    await page.getByTestId("crumb-home").click();
    await expect(page.getByTestId("page-home")).toBeVisible();
    await page.getByTestId("home-org").click();
    await expect(page).toHaveURL(/\/org\/acme$/);
    await expect(page.getByTestId("org-title")).toHaveText("acme");
    await expect(page.getByTestId("org-boxes")).toBeVisible();
    await expect(page.getByTestId("org-people")).toBeVisible();
    await expect(page.getByTestId("org-access")).toBeVisible();
    await expect(page.getByTestId("page-org").locator(".dir-row")).toHaveCount(4);
    await expect(page.getByTestId("org-settings")).toBeVisible();
    await expect(page.getByTestId("page-org")).not.toContainText("Subpages");
  });

  test("alice org boxes", async ({ page }) => {
    await openOrg(page);
    await expect(page.getByTestId("who")).toContainText("alice@acme.test");
    await expect(page.getByTestId("nav-boxes")).toBeVisible();
    await expect(page.getByTestId("page-boxes")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Boxes");
    await expect(page.getByTestId("crumb-home")).toHaveText("Home");
    await expect(page.getByTestId("crumb-space")).toHaveText("acme");
    await expect(page.getByTestId("card-box-1")).toBeVisible();
    await expect(page.getByTestId("add-box")).toBeVisible();
    await expect(page.locator("table.data-table")).toHaveCount(0);
    await expect(page.getByText("None yet.")).toHaveCount(0);
    await expect(page.getByTestId("page-boxes").locator(".dir-row")).not.toHaveCount(0);
    await page.getByTestId("card-box-1").click();
    await expect(page.getByTestId("page-box")).toBeVisible();
    await expect(page.getByTestId("box-access")).toBeVisible();
    await expect(page.getByTestId("box-remove")).toBeVisible();
    await page.getByTestId("box-access").click();
    await expect(page).toHaveURL(/\/org\/acme\/boxes\/box-1\/access$/);
    await expect(page.getByTestId("page-box-access")).toBeVisible();
    await expect(page.getByTestId("box-access-list")).toContainText("alice");
    await expect(page.getByTestId("box-grant")).toBeVisible();
    await page.getByTestId("box-grant").click();
    await expect(page).toHaveURL(/\/org\/acme\/boxes\/box-1\/access\/new$/);
    await expect(page.getByTestId("page-box-grant")).toBeVisible();
    await expect(page.getByTestId("grant-person")).toBeVisible();
    await expect(page.getByTestId("grant-submit")).toBeVisible();
  });

  test("people is membership only", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("nav-people").click();
    await expect(page).toHaveURL(/\/org\/acme\/people$/);
    await expect(page.getByTestId("page-people")).toBeVisible();
    await expect(page.getByTestId("add-person")).toBeVisible();
    await expect(page.locator("table.data-table")).toHaveCount(0);
    await expect(page.getByText("None yet.")).toHaveCount(0);
    await expect(page.getByTestId("card-alice")).toBeVisible();
    await expect(page.getByTestId("page-people")).not.toContainText("online");
    await expect(page.getByTestId("page-people")).not.toContainText("offline");
    await expect(page.getByText("Can use")).toHaveCount(0);
    await page.getByTestId("add-person").click();
    await expect(page).toHaveURL(/\/org\/acme\/people\/new$/);
    await expect(page.getByTestId("page-add-person")).toBeVisible();
    await expect(page.getByTestId("person-email")).toBeVisible();
    await expect(page.getByTestId("person-name")).toHaveCount(0);
    const email = `eve-${Date.now().toString(36)}@acme.test`;
    const handle = email.split("@")[0].replace(/[^a-z0-9]/g, "");
    await page.getByTestId("person-email").fill(email);
    await page.getByTestId("add-submit").click();
    await expect(page.getByTestId("secret")).toContainText("Token", { timeout: 20_000 });
    await page.getByTestId("done").click();
    await expect(page.getByTestId(`card-${handle}`)).toBeVisible();
    await page.getByTestId("card-alice").click();
    await expect(page).toHaveURL(/\/org\/acme\/people\/alice$/);
    await expect(page.getByTestId("page-person")).toBeVisible();
    await expect(page.getByTestId("person-role")).toBeVisible();
    await expect(page.getByTestId("person-remove")).toBeVisible();
    await expect(page.getByTestId("person-box")).toHaveCount(0);
    await page.getByTestId("person-remove").click();
    await expect(page).toHaveURL(/\/org\/acme\/people\/alice\/remove$/);
    await expect(page.getByTestId("page-person-remove")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Remove person");
  });

  test("access is a list page, grant is its own page", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("nav-access").click();
    await expect(page).toHaveURL(/\/org\/acme\/access$/);
    await expect(page.getByTestId("page-access")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Access");
    await expect(page.getByTestId("access-table")).toBeVisible();
    await expect(page.getByTestId("access-table")).not.toContainText("all boxes");
    await expect(page.getByText("None yet.")).toHaveCount(0);
    await expect(page.getByText("No grants yet.")).toHaveCount(0);
    await expect(page.getByTestId("grant-access")).toBeVisible();
    await expect(page.getByTestId("grant-person")).toHaveCount(0);
    await page.getByTestId("grant-access").click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/new$/);
    await expect(page.getByTestId("page-grant")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Grant access");
    await expect(page.getByTestId("grant-person")).toBeVisible();
    await expect(page.getByTestId("grant-box")).toBeVisible();
    await expect(page.getByTestId("grant-submit")).toBeVisible();
  });

  test("grant from a person returns to that person", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("nav-access").click();
    await page.locator("[data-testid='access-table'] a").filter({ hasText: /^alice$/ }).click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/alice$/);
    await expect(page.getByTestId("page-access-person")).toBeVisible();
    await page.getByTestId("grant-access").click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/alice\/new$/);
    await expect(page.getByTestId("page-grant")).toBeVisible();
    await page.getByTestId("grant-submit").click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/alice$/);
    await expect(page.getByTestId("page-access-person")).toBeVisible();
  });

  test("add box is its own page", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("add-box").click();
    await expect(page.getByTestId("page-add-box")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Add box");
    await expect(page.getByTestId("page-add-box")).not.toContainText("Lowercase, like box-1");
    await expect(page.getByTestId("page-add-box")).not.toContainText("Letters, digits, hyphens.");
    await expect(page.getByTestId("add-submit")).toHaveText("Add box");
    const name = `box-e2e-${Date.now().toString(36)}`;
    await page.getByTestId("box-name").fill(name);
    await page.getByTestId("add-submit").click();
    await expect(page.getByTestId("secret")).toContainText("Token", { timeout: 20_000 });
    await page.getByTestId("done").click();
    await expect(page.getByTestId(`card-${name}`)).toBeVisible();
  });

  test("personal add box has no format hint", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await page.getByTestId("home-personal").click();
    await page.getByTestId("org-boxes").click();
    await page.getByTestId("add-box").click();
    await expect(page).toHaveURL(/\/personal\/boxes\/new$/);
    await expect(page.getByTestId("page-add-box")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Add box");
    await expect(page.getByTestId("crumb-home")).toHaveText("Home");
    await expect(page.getByTestId("crumb-space")).toHaveText("Personal");
    await expect(page.getByTestId("page-add-box")).not.toContainText("Lowercase, like box-1");
    await expect(page.getByTestId("page-add-box")).not.toContainText("Letters, digits, hyphens.");
    await expect(page.getByTestId("add-submit")).toHaveText("Add box");
  });

  test("theme toggles light and dark", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("theme")).toBeVisible();
    const before = await page.locator("html").getAttribute("data-theme");
    await page.getByTestId("theme").click();
    const after = await page.locator("html").getAttribute("data-theme");
    expect(after).not.toBe(before);
  });

  test("profile lets you set a display name", async ({ page }) => {
    await signIn(page);
    await page.getByTestId("who").click();
    await expect(page).toHaveURL(/\/profile$/);
    await expect(page.getByTestId("page-profile")).toBeVisible();
    await expect(page.getByTestId("page-profile")).toContainText("alice@acme.test");
    await page.getByTestId("profile-name").fill("Alice Acme");
    await page.getByTestId("profile-save").click();
    await expect(page.getByTestId("profile-saved")).toBeVisible();
    await page.getByTestId("nav-home").click();
    await expect(page.getByTestId("page-home")).toBeVisible();
    await page.getByTestId("who").click();
    await expect(page.getByTestId("profile-name")).toHaveValue("Alice Acme");
  });

  test("org settings can lock default access", async ({ page }) => {
    await signIn(page);
    await page.getByTestId("home-org").click();
    await page.getByTestId("org-settings").click();
    await expect(page).toHaveURL(/\/org\/acme\/settings$/);
    await expect(page.getByTestId("page-settings")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Settings");
    await expect(page.getByTestId("page-settings")).not.toContainText(
      "Who may App to a box before anyone is assigned.",
    );
    await expect(page.getByTestId("org-open")).toBeVisible();
    await expect(page.getByTestId("org-open")).toContainText("Full access");
    await expect(page.getByTestId("org-open")).toContainText("No access");
    await expect(page.getByTestId("org-open")).not.toContainText("Everyone, until you assign");
    await expect(page.getByTestId("org-open")).not.toContainText("No one, until you assign");
    await page.getByTestId("org-open").selectOption("lock");
    await page.getByTestId("settings-save").click();
    await expect(page.getByTestId("settings-saved")).toBeVisible();
    await page.getByTestId("nav-boxes").click();
    await page.getByTestId("card-box-1").click();
    await page.getByTestId("box-access").click();
    await expect(page.getByTestId("box-access-list")).toContainText("alice");
    await page.getByTestId("nav-settings").click();
    await page.getByTestId("org-open").selectOption("open");
    await page.getByTestId("settings-save").click();
    await expect(page.getByTestId("settings-saved")).toBeVisible();
  });
}

  test("sign out returns to sign in", async ({ page }) => {
    await signIn(page);
    await page.getByTestId("sign-out").click();
    await expect(page.getByTestId("gate")).toBeVisible();
  });

  test("csrf token is issued before sign in", async ({ page }) => {
    const r = await page.request.get("/api/auth/csrf");
    expect(r.ok()).toBeTruthy();
    const hdr = r.headers()["x-csrf-token"];
    expect(hdr).toMatch(/^[0-9a-f]{64}$/);
  });

  test("POST without csrf is forbidden", async ({ page }) => {
    const r = await page.request.post("/api/auth/login", {
      data: { email: "alice@acme.test", password: "lab-alice-8" },
      headers: { "content-type": "application/json" },
    });
    expect(r.status()).toBe(403);
  });

  test("unknown path shows not found after sign in", async ({ page }) => {
    await signIn(page);
    await page.goto("/no-such/path");
    await expect(page.getByText("Not found")).toBeVisible();
  });

  test("bad password stays on the gate", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill(EMAIL);
    await page.getByTestId("password").fill("wrong-password-99");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("gate")).toBeVisible();
    await expect(page.getByTestId("page-home")).toHaveCount(0);
    await expect(page.locator(".flash")).not.toHaveText("");
  });

  test("lab row fills email and password", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("lab-logins")).toBeVisible();
    await page.getByTestId("lab-logins").locator("tr.pick").first().click();
    await expect(page.getByTestId("email")).toHaveValue(EMAIL);
    await expect(page.getByTestId("password")).toHaveValue(PASS);
  });

  test("cara has no people, access, or settings", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await page.getByTestId("home-personal").click();
    await expect(page.getByTestId("page-org")).toBeVisible();
    await expect(page.getByTestId("nav-people")).toHaveCount(0);
    await expect(page.getByTestId("nav-access")).toHaveCount(0);
    await expect(page.getByTestId("nav-settings")).toHaveCount(0);
    await expect(page.getByTestId("org-people")).toHaveCount(0);
    await expect(page.getByTestId("org-settings")).toHaveCount(0);
  });

  test("health is ok", async ({ page }) => {
    const r = await page.request.get("/api/health");
    expect(r.ok()).toBeTruthy();
    expect((await r.text()).trim()).toBe("ok");
  });

  test("create organization page is reachable", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await page.getByTestId("home-create-org").click();
    await expect(page).toHaveURL(/\/new-organization$/);
    await expect(page.getByTestId("page-new-org")).toBeVisible();
    await expect(page.getByTestId("org-name")).toBeVisible();
    await expect(page.getByTestId("org-submit")).toBeVisible();
  });

  test("organization named personal is reserved", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await page.getByTestId("home-create-org").click();
    await page.getByTestId("org-name").fill("personal");
    await page.getByTestId("org-submit").click();
    await expect(page.getByTestId("page-new-org")).toBeVisible();
    await expect(page.locator(".flash")).toContainText("reserved");
    await expect(page).toHaveURL(/\/new-organization$/);
  });

  test("locked org shows no one on a box without grants", async ({ page }) => {
    await signIn(page);
    await page.getByTestId("home-org").click();
    await page.getByTestId("org-settings").click();
    await page.getByTestId("org-open").selectOption("lock");
    await page.getByTestId("settings-save").click();
    await expect(page.getByTestId("settings-saved")).toBeVisible();
    await page.getByTestId("nav-boxes").click();
    await expect(page.getByTestId("page-boxes")).toBeVisible();
    await expect(page.getByText("None yet.")).toHaveCount(0);
    await page.getByTestId("card-box-2").click();
    await page.getByTestId("box-access").click();
    await expect(page.getByTestId("box-access-list")).not.toHaveText("Everyone in the organization.");
    await page.getByTestId("nav-settings").click();
    await page.getByTestId("org-open").selectOption("open");
    await page.getByTestId("settings-save").click();
    await expect(page.getByTestId("settings-saved")).toBeVisible();
  });

  test("remove box from its page", async ({ page }) => {
    await openOrg(page);
    const card = page.locator("[data-testid^='card-box-e2e-']").first();
    await expect(card).toBeVisible();
    const id = await card.getAttribute("data-testid");
    const name = id?.replace("card-", "") ?? "";
    await card.click();
    await expect(page.getByTestId("page-box")).toBeVisible();
    await expect(page.getByTestId("box-remove")).toHaveText("Remove box");
    await page.getByTestId("box-remove").click();
    await expect(page.getByTestId("page-box-remove")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Remove box");
    await expect(page.getByTestId("box-remove-confirm")).toHaveText("Remove box");
    await page.getByTestId("box-remove-confirm").click();
    await expect(page.getByTestId("page-boxes")).toBeVisible();
    await expect(page.getByTestId(`card-${name}`)).toHaveCount(0);
  });

  test("add person title is Add person", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("nav-people").click();
    await expect(page.getByTestId("add-person")).toHaveText("Add person");
    await page.getByTestId("add-person").click();
    await expect(page).toHaveURL(/\/org\/acme\/people\/new$/);
    await expect(page.getByTestId("page-add-person")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Add person");
    await expect(page.getByTestId("add-submit")).toHaveText("Add person");
  });

  test("box page actions are Access and Remove box", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("card-box-1").click();
    await expect(page.getByTestId("page-box")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("box-1");
    await expect(page.getByTestId("box-access")).toContainText("Access");
    await expect(page.getByTestId("box-remove")).toHaveText("Remove box");
    await expect(page.getByTestId("box-who")).toHaveCount(0);
  });

  test("grant from list returns to access list", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("nav-access").click();
    await page.getByTestId("grant-access").click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/new$/);
    await page.getByTestId("grant-submit").click();
    await expect(page).toHaveURL(/\/org\/acme\/access$/);
    await expect(page.getByTestId("page-access")).toBeVisible();
  });

  test("allow all boxes returns to the person", async ({ page }) => {
    await openOrg(page);
    await page.getByTestId("nav-access").click();
    await page.locator("[data-testid='access-table'] a").filter({ hasText: /^alice$/ }).click();
    await expect(page.getByTestId("page-access-person")).toBeVisible();
    await page.getByTestId("access-open").click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/alice\/open$/);
    await expect(page.getByTestId("page-access-open")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Allow all boxes");
    await page.getByTestId("access-open-confirm").click();
    await expect(page).toHaveURL(/\/org\/acme\/access\/alice$/);
    await expect(page.getByTestId("page-access-person")).toBeVisible();
  });

  test("create organization title is Create organization", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await page.getByTestId("home-create-org").click();
    await expect(page.getByTestId("page-new-org")).toBeVisible();
    await expect(page.getByTestId("org-title")).toHaveText("Create organization");
    await expect(page.getByTestId("org-submit")).toHaveText("Create organization");
  });

  test("personal box has no grant access", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("email").fill("cara@home.test");
    await page.getByTestId("password").fill("lab-cara-8x");
    await page.getByTestId("submit").click();
    await expect(page.getByTestId("page-home")).toBeVisible({ timeout: 20_000 });
    await page.getByTestId("home-personal").click();
    await page.getByTestId("org-boxes").click();
    const card = page.locator("[data-testid^='card-']").first();
    if ((await card.count()) === 0) {
      return;
    }
    await card.click();
    await expect(page.getByTestId("page-box")).toBeVisible();
    await page.getByTestId("box-access").click();
    await expect(page.getByTestId("page-box-access")).toBeVisible();
    await expect(page.getByTestId("box-grant")).toHaveCount(0);
  });
});
