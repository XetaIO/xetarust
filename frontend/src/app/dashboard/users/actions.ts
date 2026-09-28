"use server";

import { deleteAuthorComments } from "@/features/discussion/actions";
import { banUser } from "@/features/identity/actions";
import { type FormState, optionalField } from "@/lib/forms";

/**
 * Bans the user `id` (Identity) then, when the admin ticked `delete_comments`,
 * purges their comments (Discussion). Composed here because a feature never
 * imports another one. Banning is idempotent, so a failed purge can be retried
 * by submitting the form again.
 */
export async function banMember(id: string, _: FormState, data: FormData): Promise<FormState> {
  const banned = await banUser(id, optionalField(data, "reason"));
  if (!banned?.success || data.get("delete_comments") !== "on") {
    return banned;
  }

  const purged = await deleteAuthorComments(id);
  if (!purged?.success) {
    return {
      message: `User banned, but their comments could not be deleted (${purged?.message ?? "unknown error"}). Submit again to retry.`,
    };
  }

  return { success: true, message: `User banned. ${purged.message}` };
}
