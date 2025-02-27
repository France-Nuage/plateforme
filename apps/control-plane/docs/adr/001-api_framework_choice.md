# Framework Choice for API Management

## Status

Accepted

## Context

Team size: 1 developer  
To create France-Nuage, our ideal scenario was to develop an API in RUST.  
However, the technical team, consisting of just one developer, had no prior knowledge of RUST.  
Our primary objective was to deliver the project as quickly as possible while maintaining a high-quality standard.

The technical team had extensive expertise in Laravel and Adonis, making these the most practical choices given the time constraints and the need to avoid a steep learning curve.

## Decision

AdonisJS was chosen as the framework. This decision was based on several key factors:

- **Familiarity**: The developer already had experience with AdonisJS, allowing for faster onboarding and reduced development time.
- **Language Popularity**: Being based on JavaScript, a widely-used language, AdonisJS benefits from a large ecosystem and community support.
- **Feature-Rich**: AdonisJS provides built-in functionalities, such as ORM, authentication, and validation, which reduce the need for external libraries and speed up the development process.
- **Scalability**: AdonisJS is well-suited for building scalable and maintainable APIs.

While Laravel was another viable option, the choice of AdonisJS aligned better with the team’s preference for a JavaScript-based framework and the need for a lightweight, modern solution.

## Consequences

Using AdonisJS has several advantages:

- **Efficiency**: Its comprehensive set of built-in tools minimizes development time.
- **Streamlined Workflow**: AdonisJS provides a structured and opinionated framework, making it easier to organize and maintain the codebase.

However, like many modern frameworks, AdonisJS operates as a "black box," which can occasionally make debugging and customization more challenging. To mitigate this, it is essential to maintain clear documentation and rely on the community for troubleshooting.
