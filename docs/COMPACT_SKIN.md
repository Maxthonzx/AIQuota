# Compact skin

Choose **Settings → Interface skin → Compact** to see more accounts at once. Classic remains the default and can be restored at any time. Skin and light/dark/system theme preferences are independent; switching does not change connections, credentials or quota data.

The compact skin uses green colors, top navigation, three columns at 1120px and two at 640px. Each quota window shows the countdown on the left and the reset date and time on separate lines on the right, in the computer's local timezone.

The implementation shares the existing React data and action components. It introduces no demo accounts, backend changes or new dependencies. `src/skin.ts` stores the preference and `src/compact.css` scopes the skin styles. Reverting the change requires no data migration.

Validation: six frontend tests, source lint and production build passed. Browser checks used isolated fictional native-command responses to verify theme and skin changes, persistence, provider list, configuration dialog, and 1120×800 / 640×480 layouts. Native installation, real account queries and Windows WebView2 were not tested locally; GitHub Actions builds the desktop packages.
