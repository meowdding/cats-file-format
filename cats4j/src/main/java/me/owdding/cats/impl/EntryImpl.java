package me.owdding.cats.impl;

import me.owdding.cats.api.CatsEntry;
import me.owdding.cats.api.CatsException;
import me.owdding.cats.api.compression.CatsCompression;
import me.owdding.cats.api.compression.CatsCompressions;

import java.io.IOException;
import java.util.HashMap;
import java.util.Map;
import java.util.regex.Pattern;

public final class EntryImpl {

    public static CatsEntry.Directory read(DataReader reader) throws IOException {
        return Directory.read(reader, "/");
    }

    private static CatsEntry read(byte type, DataReader reader, String prefix) throws IOException {
        return switch (type) {
            case 0x00 -> File.read(reader);
            case 0x01 -> Directory.read(reader, prefix);
            default -> throw new CatsException("Unknown entry type: " + type);
        };
    }

    private record File(int offset, int size, CatsCompression compression) implements CatsEntry.File {

        private static final int HEADER_SIZE = Integer.BYTES + Integer.BYTES + Byte.BYTES;

        @Override
        public int header() {
            return HEADER_SIZE;
        }

        private static CatsEntry.File read(DataReader reader) throws IOException {
            int offset = reader.readInt();
            int size = reader.readInt();
            byte compression = reader.readByte();
            return new EntryImpl.File(
                    offset,
                    size,
                    switch (compression) {
                        case CatsCompressions.KEY_NONE -> CatsCompressions.NONE;
                        case CatsCompressions.KEY_GZIP -> CatsCompressions.GZIP;
                        default -> throw new CatsException("Unknown compression method: " + Byte.toUnsignedInt(compression));
                    }
            );
        }
    }

    private record Directory(
            int header,
            Map<String, CatsEntry> entries
    ) implements CatsEntry.Directory {

        // All printable ascii except space, / and \
        public static final Pattern VALID_NAME = Pattern.compile("[\\x20-\\x2E\\x30-\\x5B\\x5D-\\x7E]+");

        private static CatsEntry.Directory read(DataReader reader, String prefix) throws IOException {
            int size = Short.BYTES;
            int count = reader.readUnsignedShort();
            Map<String, CatsEntry> entries = new HashMap<>();

            for (int i = 0; i < count; i++) {
                var type = reader.readByte();
                var name = reader.readString();
                var entry = EntryImpl.read(type, reader, prefix + name + "/");
                var suffix = entry instanceof EntryImpl.Directory ? "/" : "";

                if (!VALID_NAME.matcher(name).matches()) {
                    throw new CatsException("Invalid entry name: '" + name + "'");
                }

                entries.put(String.format("%s%s%s", prefix, name, suffix), entry);

                size += Byte.BYTES + Byte.BYTES + name.length() + entry.header();
            }

            return new EntryImpl.Directory(size, entries);
        }
    }
}
