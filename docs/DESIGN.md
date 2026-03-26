# Design System Strategy: The Kinetic Engine

## 1. Overview & Creative North Star: "The Kinetic Engine"
This design system is built for the high-performance developer. It moves away from the "SaaS-template" look and toward a **Kinetic Engine** aesthetic—an environment that feels like a precision-engineered instrument. Our Creative North Star focuses on the intersection of high-speed performance and editorial sophistication. 

We achieve this by breaking the rigid, boxed-in layouts of standard dashboards. Instead, we utilize intentional asymmetry, high-contrast typography scales, and overlapping "glass" layers. The interface should feel like a high-end terminal—dark, focused, and undeniably fast. We don't just show data; we showcase power through tonal depth and typographic authority.

## 2. Colors: Tonal Depth & The "No-Line" Rule
The color palette is rooted in a deep obsidian foundation (`background`: `#0f1419`), punctuated by high-energy electric pulses (`primary`: `#adc6ff`).

*   **The "No-Line" Rule:** We do not use 1px solid borders to define sections. Traditional borders create visual noise that slows down the user's eye. Instead, boundaries must be defined solely through background color shifts. A `surface-container-low` (`#171c21`) section sitting on a `surface` (`#0f1419`) background is enough to signify a change in context.
*   **Surface Hierarchy & Nesting:** Treat the UI as a physical stack of materials. 
    *   **Base:** `surface` (#0f1419)
    *   **Low-Level Sections:** `surface-container-low` (#171c21)
    *   **Interactive Cards:** `surface-container` (#1b2025)
    *   **Popovers/Floating Elements:** `surface-container-highest` (#30353a)
*   **The "Glass & Gradient" Rule:** To signify innovation, floating panels (modals, tooltips, or navigation) should utilize Glassmorphism. Use semi-transparent versions of `surface-variant` with a `backdrop-blur` of 12px–20px. 
*   **Signature Textures:** For main CTAs and Hero moments, avoid flat fills. Use a subtle linear gradient from `primary` (#adc6ff) to `primary_container` (#0969da) at a 135-degree angle to give the element "weight" and kinetic energy.

## 3. Typography: Editorial Precision
The system leverages a dual-font strategy to balance human-centric clarity with technical grit.

*   **Display & Headlines:** We use **Mona Sans VF** (via `display` and `headline` tokens). This should be set with tight letter-spacing (-2% to -4%) and high weights. Use large scales (`display-lg`: 3.5rem) to create editorial "anchor points" on the page.
*   **Technical Details & Labels:** **ui-monospace** (via `label` and technical body tokens) is our workhorse. Use it for any data points, code snippets, or metadata. It signals to the developer that the information is precise and "system-level."
*   **Hierarchy as Brand:** By pairing a massive, bold Mona Sans headline with a tiny, monospace `label-sm` (#0.6875rem), we create a high-contrast "pro-tool" look that feels both premium and functional.

## 4. Elevation & Depth: Tonal Layering
We do not use shadows to simulate height; we use light and material density.

*   **The Layering Principle:** Depth is achieved by stacking `surface-container` tiers. Place a `surface-container-lowest` card inside a `surface-container-high` section to create a "recessed" look, or vice versa for a "lifted" look.
*   **Ambient Shadows:** If an element must float (e.g., a dropdown), use an extra-diffused shadow.
    *   **Blur:** 24px–40px.
    *   **Opacity:** 6%.
    *   **Color:** Use a tinted version of `surface_tint` (#adc6ff) rather than pure black to simulate the glow of a screen.
*   **The "Ghost Border" Fallback:** If a container requires a border for accessibility, use a "Ghost Border." Apply `outline-variant` (#424753) at 15% opacity. It should be felt, not seen.

## 5. Components

### Buttons
*   **Primary:** Gradient fill (`primary` to `primary_container`), `on_primary` text. Use `rounded-sm` (0.125rem) for a sharp, technical edge.
*   **Tertiary:** Ghost style. No background, `primary` text, and a `surface-variant` background on hover.

### Input Fields
*   **Styling:** Use `surface_container_low` for the fill. 
*   **Typography:** The input text must always use `ui-monospace`.
*   **Focus State:** Instead of a thick border, use a 2px "Glow" on the bottom edge using the `primary` token.

### Cards & Lists
*   **The Divider Forbid:** Never use `<hr>` or border-bottom to separate list items. Use vertical white space (`spacing-4` or `spacing-6`) or a subtle hover state shift to `surface_container_highest`.
*   **Layout:** Use asymmetrical padding (e.g., more padding on the left than the right) to break the "standard" grid and create a custom, editorial feel.

### Terminal Chips
*   A custom component for technical tags. Use `ui-monospace`, `label-sm`, and a `secondary_container` background with `on_secondary_container` text. Keep corners sharp (`rounded-none`).

## 6. Do’s and Don’ts

### Do
*   **Do** use monospace for all numbers, timestamps, and IDs.
*   **Do** allow elements to overlap (e.g., a floating code snippet overlapping a headline) to create depth.
*   **Do** use the `spacing-20` and `spacing-24` tokens to create "Hero" breathing room.
*   **Do** use `primary_fixed_dim` for subtle accents that shouldn't scream for attention.

### Don’t
*   **Don’t** use 100% opaque borders. They clutter the "pro" aesthetic.
*   **Don’t** use standard "drop shadows" (0, 2, 4, etc.). They look amateur in a high-performance system.
*   **Don’t** use vibrant accent colors for large background areas. Keep them for "pulses" and "indicators."
*   **Don’t** use `body-md` for technical data; always default to `label-md` (monospace).