Here's the project structure:

- **`models/order.ts`** — `Order` interface with status: `'pending' | 'confirmed' | 'shipped' | 'delivered' | 'cancelled'`
- **`services/order-service.ts`** — create/get orders using the db wrapper
- **`utils/db.ts`** — stub db with `orders` and `notifications` collections

Here's the exact code for `src/services/notification-service.ts`:

```ts
import { db } from '../utils/db';
import { Order } from '../models/order';

type NotifiableStatus = Exclude<Order['status'], 'pending'>;

interface NotificationChannel {
  send(to: string, subject: string, body: string): Promise<void>;
}

const statusMessages: Record<NotifiableStatus, { subject: string; body: (orderId: string) => string }> = {
  confirmed: {
    subject: 'Order Confirmed',
    body: (id) => `Your order ${id} has been confirmed and is being prepared.`,
  },
  shipped: {
    subject: 'Order Shipped',
    body: (id) => `Your order ${id} has been shipped.`,
  },
  delivered: {
    subject: 'Order Delivered',
    body: (id) => `Your order ${id} has been delivered.`,
  },
  cancelled: {
    subject: 'Order Cancelled',
    body: (id) => `Your order ${id} has been cancelled.`,
  },
};

class EmailChannel implements NotificationChannel {
  async send(to: string, subject: string, body: string): Promise<void> {
    // Replace with real email provider (SES, SendGrid, etc.)
    console.log(`Email to ${to}: [${subject}] ${body}`);
  }
}

export class NotificationService {
  private channels: NotificationChannel[];

  constructor(channels: NotificationChannel[]) {
    this.channels = channels;
  }

  async notifyOrderStatusChange(order: Order, status: NotifiableStatus): Promise<void> {
    const message = statusMessages[status];
    const body = message.body(order.id);

    await Promise.all(
      this.channels.map((ch) => ch.send(order.userId, message.subject, body)),
    );

    await db.notifications.create({
      orderId: order.id,
      userId: order.userId,
      type: 'order_status',
      status,
      createdAt: new Date(),
    });
  }
}

export function createNotificationService(): NotificationService {
  return new NotificationService([new EmailChannel()]);
}
```

**Design decisions:**

- **`NotificationChannel` interface** — Adding SMS/push later means implementing the interface and passing it to the constructor. No changes to `NotificationService` itself.
- **`NotifiableStatus`** — Derived from `Order['status']` excluding `'pending'` (initial state, no notification). If the Order type changes, this stays in sync automatically.
- **`statusMessages` map** — Keeps message templates out of the send logic. The `Record<NotifiableStatus, ...>` type ensures every notifiable status has a message — a compile error if one is missing.
- **`db.notifications.create`** — Persists a log of sent notifications using the existing db stub, so you have an audit trail regardless of which channels fired.
- **`createNotificationService` factory** — Wires up channels in one place. When you add SMS, you add it here.

To integrate, you'd call it from the order service when updating status:

```ts
const notifications = createNotificationService();
// after updating order status:
await notifications.notifyOrderStatusChange(order, 'shipped');
```
