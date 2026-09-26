import Image from "next/image";

/** Xetaravel brand mark. */
export function Logo() {
    return (
        <span className="inline-flex items-center gap-2 text-lg">
            <Image src="/images/logo.svg" alt="" width={20} height={20} priority />
            <span>Xetaravel</span>
        </span>
    );
}
