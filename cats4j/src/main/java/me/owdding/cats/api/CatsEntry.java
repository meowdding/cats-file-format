package me.owdding.cats.api;

import me.owdding.cats.api.compression.CatsCompression;

import java.util.Map;

public sealed interface CatsEntry {

    /**
     * @return The size of the entry itself in bytes. Not including actual file data.
     */
    int header();

    non-sealed interface File extends CatsEntry {

        /**
         * @return The offset of the file data in the archive from the header.
         */
        int offset();

        /**
         * @return The size of the file data in bytes.
         */
        int size();

        /**
         * @return The compression method used for this file.
         */
        CatsCompression compression();
    }

    non-sealed interface Directory extends CatsEntry {

        /**
         * @return A map of entry names to their corresponding CatsEntry objects.
         */
        Map<String, CatsEntry> entries();
    }
}
