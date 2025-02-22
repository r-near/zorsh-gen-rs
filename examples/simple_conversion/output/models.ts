import { b } from '@zorsh/zorsh';

export const PlayerStatusSchema = b.enum({
    Online: b.unit(),
    Offline: b.struct({
        last_seen: b.u64()
    }),
    Away: b.string()
});
export type PlayerStatus = b.infer<typeof PlayerStatusSchema>;

export const ItemSchema = b.struct({
    id: b.u32(),
    name: b.string(),
    attributes: b.hashMap(b.string(), b.u32())
});
export type Item = b.infer<typeof ItemSchema>;

export const InventorySchema = b.struct({
    items: b.vec(ItemSchema),
    capacity: b.u32()
});
export type Inventory = b.infer<typeof InventorySchema>;

export const PlayerSchema = b.struct({
    name: b.string(),
    level: b.u8(),
    experience: b.u32(),
    inventory: InventorySchema,
    status: b.option(PlayerStatusSchema),
    achievements: b.vec(b.string())
});
export type Player = b.infer<typeof PlayerSchema>;

export const GameStateSchema = b.struct({
    players: b.hashMap(b.string(), PlayerSchema),
    current_round: b.u32(),
    timestamp: b.u64()
});
export type GameState = b.infer<typeof GameStateSchema>;

