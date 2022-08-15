/// <reference types="@sveltejs/kit" />

declare namespace App {
    interface Locals {
        user: {
            id: string,
            username: string
        }
    }

    interface Platform {}

    interface Session {}

    interface Stuff {}
}
