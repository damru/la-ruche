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