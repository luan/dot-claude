You already have `isValidEmail` in `src/components/signup-form.ts:3`. I'd recommend reusing that rather than writing a new one.

Import it in your API route:

```ts
import { isValidEmail } from '../components/signup-form';
```

Then add your signup route with validation:

```ts
router.post('/signup', (req, res) => {
  const { email } = req.body;

  if (!email || typeof email !== 'string' || !isValidEmail(email.trim())) {
    return res.status(400).json({ error: 'Invalid email address' });
  }

  // process signup...
});
```

One consideration: `isValidEmail` lives in a component file, which is a bit odd for a utility used by the API layer. If you want cleaner separation, you could move the function and regex to something like `src/lib/validation.ts` and have both the component and the route import from there. But that's a refactor choice — functionally, the existing validation is solid.
