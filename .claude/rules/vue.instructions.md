---
description: 'Vue.js coding conventions and best practices'
applyTo: '**/*.vue'
---

# Vue.js Coding Conventions and Best Practices



## General Instructions

- Always prioritize readability, safety, and maintainability.
- Use strong typing and leverage TypeScript's type system for safety.
- Never implement functional methods directly in the template — all logic must be in the backend. Just display data as-is or do some simple calculations. 
- Write code with good maintainability practices, including comments on why certain design decisions were made.
- Handle forms feedback (success and errors) gracefully and provide meaningful error messages.

## Error Handling

- Provide meaningful error messages to users, especially for form validation errors. Use the `field-hint` class for additional guidance on form fields.
- For API errors, display user-friendly messages and log technical details for debugging.

## Design Guidelines

- Always create or reuse SVG icons instead of using icon fonts or external libraries. This ensures visual consistency and optimal performance. These icons are located as atomic components in the `src/components/atoms/` directory.
- Follow the design system tokens for typography, colors, and spacing as defined in `DESIGN.md`. Use CSS variables for colors and font weights to maintain consistency across the application.

## Form Feedback Pattern (DDR-006 — Non-negotiable)

**Never use `CFormFeedback` directly in templates except if specifically asked.** Always use `CFormActions` (modal footer) or `CForm` (inline form). These molecules handle layout and the push animation internally.

- **Modal forms**: wrap all buttons in `CFormActions` inside the `#footer` slot — never put `CFormFeedback` in the modal body.
- **Delete button on the left**: add `class="mr-auto"` to the first `CBtn` inside `CFormActions`.
- **Inline forms**: use `CForm` with an `#actions` slot instead of a bare `<form>`.
- Use `useFormFeedback()` for state; call `feedback.reset()` when opening a modal.