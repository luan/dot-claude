That's it. Here's the core of it — `signup.ts`:

```typescript
export async function signup(request: SignupRequest): Promise<User> {
  const user = await createUser(request);

  await sendWelcomeEmail(user);
  await createAuditLog({ action: "user.signup", userId: user.id });
  await trackSignup(user);

  return user;
}
```

Three function calls in sequence. No event bus, no observer pattern, no subscriber registry. You said these three things only happen on signup and nowhere else — so a direct call is the simplest, most readable, and most maintainable approach. You can read the signup function and see exactly what happens, in order.

If email/analytics failures shouldn't block signup, you'd wrap those in try/catch or fire them with `Promise.allSettled` — but that's a resilience decision, not an architecture one.
