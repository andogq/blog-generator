/// <reference types="@sveltejs/kit" />

declare namespace App {
    interface Locals {
        user: import("@prisma/client").User | null
    }

    interface Platform {}

    interface Session {}

    interface Stuff {}
}
