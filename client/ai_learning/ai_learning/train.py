import argparse
import random
import yaml
import numpy as np
import math
import json
import os
import csv
import pandas as pd
import matplotlib.pyplot as plt
import copy
# from multiprocessing import Pool

TILE_SPAWN_DENSITY = {
    "food": 0.5,
    "linemate": 0.3,
    "deraumere": 0.15,
    "sibur": 0.1,
    "mendiane": 0.1,
    "phiras": 0.08,
    "thystame": 0.05
}

RESOURCE_KEYS = list(TILE_SPAWN_DENSITY.keys())

STONE_KEYS = [r for r in RESOURCE_KEYS if r != "food"]

EVOLUTION_TABLE = {
    # level: [nb_players linemate deraumere sibur mendiane phiras thystame]
    1: [1, 1, 0, 0, 0, 0, 0],
    2: [2, 1, 1, 1, 0, 0, 0],
    3: [2, 2, 0, 1, 0, 2, 0],
    4: [4, 1, 1, 2, 0, 1, 0],
    5: [4, 1, 2, 1, 3, 0, 0],
    6: [6, 1, 2, 3, 0, 1, 0],
    7: [6, 2, 2, 2, 2, 2, 1]
}

MAX_LEVEL = 8.0
MAX_PLAYERS = 6.0
MAX_STONE_REQ = 3.0

DIRECTION_VECTORS = {
    0: (0, -1),  # North
    1: (1,  0),  # East
    2: (0,  1),  # South
    3: (-1, 0),  # West
}

LEFT_ROT = {
    1: 7,
    2: 8,
    3: 1,
    4: 2,
    5: 3,
    6: 4,
    7: 5,
    8: 6,
}

RIGHT_ROT = {
    1: 3,
    2: 4,
    3: 5,
    4: 6,
    5: 7,
    6: 8,
    7: 1,
    8: 2,
}

BROADCAST_TABLE = {
    0: [7, 8, 1, 2, 3, 4, 5, 6],  # NORTH
    1: [1, 2, 3, 4, 5, 6, 7, 8],  # EAST
    2: [3, 4, 5, 6, 7, 8, 1, 2],  # SOUTH
    3: [5, 6, 7, 8, 1, 2, 3, 4],  # WEST
}

ACTIONS = {
    "INCANTATE":       0,
    "PUSH":            1,
    "MOVE_FORWARD":    2,
    "TURN_LEFT":       3,
    "TURN_RIGHT":      4,
    "TAKE_FOOD":       5,
    "TAKE_LINEMATE":   6,
    "TAKE_DERAUMERE":  7,
    "TAKE_SIBUR":      8,
    "TAKE_MENDIANE":   9,
    "TAKE_PHIRAS":    10,
    "TAKE_THYSTAME":  11,
    "DROP_FOOD":      12,
    "DROP_LINEMATE":  13,
    "DROP_DERAUMERE": 14,
    "DROP_SIBUR":     15,
    "DROP_MENDIANE":  16,
    "DROP_PHIRAS":    17,
    "DROP_THYSTAME":  18,
    "EAT":            19,
    "FORK":           20,
    "BROADCAST_ADV":  21,
    "BROADCAST_INV":  22,
}

MAX_VISION_TILES = 64 # lvl 7 so 7 rows (we stop once lvl 8 get so don't need to calculate it)

def fork_agent(parent_genome, mutation_std=0.01):
    return parent_genome + np.random.randn(len(parent_genome)) * mutation_std

def compute_sound_direction(sender, receiver, map_size):
    if sender.coordinates == receiver.coordinates:
        return 0

    w, h = map_size

    dx = sender.coordinates[0] - receiver.coordinates[0]
    dy = sender.coordinates[1] - receiver.coordinates[1]

    if dx > w / 2:
        dx -= w
    elif dx < -w / 2:
        dx += w

    if dy > h / 2:
        dy -= h
    elif dy < -h / 2:
        dy += h

    angle = math.degrees(math.atan2(-dy, dx))
    angle = (angle + 360) % 360

    sector = round(angle / 45) % 8

    return BROADCAST_TABLE[receiver.direction][sector]

def compute_adv(agent, agents, tile):
    needed = EVOLUTION_TABLE[agent.level]

    players_around = [a for a in agents
        if a.coordinates == agent.coordinates
        and a.level == agent.level
        and a.survival_ticks > 0]

    missing_resources = [
        max(0, needed[i + 1] - tile[key])
        for i, key in enumerate(STONE_KEYS)
    ]
    return max(0, needed[0] - len(players_around)), missing_resources

def compare_adv(missing_players_1, missing_stone_1, missing_players_2, missing_stone_2, level):
    needed = EVOLUTION_TABLE[level]
    max_players = needed[0]
    max_stones  = sum(needed[1:])

    score_1 = missing_players_1 / max(max_players, 1) + sum(missing_stone_1) / max(max_stones, 1)
    score_2 = missing_players_2 / max(max_players, 1) + sum(missing_stone_2) / max(max_stones, 1)
    return score_1 >= score_2


class Broadcast:
    def __init__(
        self,
        msg_type,
        level,
        players_needed,
        missing_stones
    ):
        self.msg_type = msg_type
        self.level = level
        self.players_needed = players_needed
        self.missing_stones = missing_stones
        self.direction = 0

    @staticmethod
    def create_msg(agent, players_needed, missing_resources, msg_type):
        if msg_type == "adv":
            return Broadcast(
                msg_type,
                agent.level,
                players_needed,
                missing_resources
            )
        if msg_type == "inv":
            return Broadcast(
                msg_type,
                agent.level,
                1,
                [agent.inventory[r] for r in STONE_KEYS]
            )
    
    def normalize(self, broadcast):
        broadcast[0] = 1.0
        broadcast[1 + self.direction] = 1.0

        broadcast[10] = self.level / MAX_LEVEL
        broadcast[11] = self.players_needed / MAX_PLAYERS
        for i, val in enumerate(self.missing_stones):
            broadcast[12 + i] = val / MAX_STONE_REQ
        broadcast[18] = 1.0 if self.msg_type == "adv" else 0.0
        broadcast[19] = 1.0 if self.msg_type == "inv" else 0.0
        return broadcast


class ZappyEnv:
    def __init__(self, map_size, episode_ticks, max_team_size):
        self.tile_number = map_size[0] * map_size[1]
        self.map = [
            {resource: 0 for resource in RESOURCE_KEYS}
            for _ in range(map_size[0] * map_size[1])
        ]
        self.map_size = map_size
        self.episode_ticks = episode_ticks
        self.max_team_size = max_team_size

        self.nb_fork = 0
        self.nb_incantation = 0
        self.nb_broadcast = 0
        self.nb_pos_broadcast = 0
        self.nb_neg_broadcast = 0
        self.useful_drops = 0
        self.useless_drops = 0
        self.objective_prog = 0
        self.objective_failed = 0
        self.teammates_reach = 0

    def update(self):
        for resource, density in TILE_SPAWN_DENSITY.items():
            target = max(int(self.tile_number * density), 1)

            current = sum(tile[resource] for tile in self.map)

            missing = target - current

            for _ in range(max(0, missing)):
                idx = np.random.randint(0, self.tile_number)
                self.map[idx][resource] += 1

    def compute_valid_mask(self, agent, team_agents, other_team):
        mask = np.ones(len(ACTIONS), dtype=bool)

        tile = self.map[agent.coordinates[0] + agent.coordinates[1] * self.map_size[0]]

        needed = EVOLUTION_TABLE[agent.level]
        agents_on_tile = [a for a in (team_agents + other_team)
                if a.coordinates == agent.coordinates
                and a.level == agent.level
                and a is not agent
                and a.survival_ticks > 0]
        stones_ok = all(tile[r] >= req
                for r, req in zip(STONE_KEYS, needed[1:]))
        mask[ACTIONS["INCANTATE"]] = (len(agents_on_tile) + 1 >= needed[0]) and stones_ok

        ennemies_on_tile = [a for a in other_team
                if a.coordinates == agent.coordinates]
        mask[ACTIONS["PUSH"]] = len(ennemies_on_tile) >= 1

        for i, resource in enumerate(RESOURCE_KEYS):
            mask[5 + i] = tile[resource] > 0

        for i, resource in enumerate(RESOURCE_KEYS):
            mask[12 + i] = agent.inventory[resource] > 0

        mask[ACTIONS["EAT"]] = agent.inventory["food"] > 0

        mask[ACTIONS["FORK"]] = len(team_agents) < self.max_team_size

        return mask

    def run(self, agents_a, agents_b):
        teams = [
            (agents_a, agents_b, 0),
            (agents_b, agents_a, 1)
        ]

        for tick in range(self.episode_ticks):
            if tick % 20 == 0:
                self.update()
            dead_agent = 0

            for team, other_team, winner in teams:
                for agent in team:
                    if not agent.update(tick):
                        dead_agent += 1
                        continue

                    if agent.current_objective is not None:
                        agent.objective_age += 1
                        if agent.objective_age > agent.max_age:
                            agent.current_objective = None
                            agent.objective_failed += 1
                            self.objective_failed += 1

                    state = agent.get_state(self, team, other_team)
                    valid_mask = self.compute_valid_mask(agent, team, other_team)
                    action = agent.act(state, valid_mask)
                    self.apply_action(agent, action, team, other_team)

                    if agent.level == 8:
                        return winner
            if dead_agent == len(agents_a) + len(agents_b):
                return -1
        return -1

    def apply_action(self, agent, action, team_agents, other_team):
        """
        action 0 is incantate / elevate
        action 1 is push
        action 2 is move forward
        action 3 to 4 is turn 0 left, 1 right
        action 5 to 11 is get a ressoure (in order of TILE_SPAWN_DENSITY keys)
        action 12 to 18 is drop a ressoure (in order of TILE_SPAWN_DENSITY keys)
        action 19 is eating
        action 20 is giving birth
        action 21 is broadcasting
        """
        tile = self.map[agent.coordinates[0] + agent.coordinates[1] * self.map_size[0]]
        if action == 0:
            needed = EVOLUTION_TABLE[agent.level]
            
            agents_on_tile = [a for a in (team_agents + other_team)
                            if a.coordinates == agent.coordinates
                            and a.level == agent.level
                            and a.survival_ticks > 0]

            for resource, req in zip(STONE_KEYS, needed[1:]):
                tile[resource] -= req
            for a in agents_on_tile:
                a.level += 1
            agent.incantation += 1
            self.nb_incantation += 1
        elif action == 1:
            for a in other_team:
                if a.coordinates == agent.coordinates:
                    a.move(-1, agent.direction)
        elif action == 2:
            if agent.move(1):
                self.objective_prog += 1
            players_around = [a for a in team_agents
                if a.coordinates == agent.coordinates
                and a.level == agent.level
                and a.survival_ticks > 0]
            if players_around:
                self.teammates_reach += 1
                agent.current_objective = None
                agent.objective_progress += 2

        elif 3 <= action <= 4:
            if agent.turn(action - 3):
                self.objective_prog += 1

        elif 5 <= action <= 11:
            resource = list(RESOURCE_KEYS)[action - 5]
            tile[resource] -= 1
            agent.get_resource(resource)

        elif 12 <= action <= 18:
            resource = list(RESOURCE_KEYS)[action - 12]
            tile[resource] += 1
            if agent.drop_resource(resource, tile):
                self.useful_drops += 1
            else:
                self.useless_drops += 1

        elif action == 19:
            agent.eat()

        elif action == 20:
            team_agents.append(Agent(fork_agent(agent.genome), agent.layer_dims, self.map_size))
            team_agents[-1].move_to(agent.coordinates)
            self.nb_fork += 1

        else:
            self.nb_broadcast += 1
            players_needed, missing_resources = compute_adv(agent, (team_agents + other_team), tile)
            useless = players_needed + sum(missing_resources) == 0

            if action == 21:
                BROADCAST_TYPE = "adv"
                if useless:
                    agent.neg_broadcast += 2
                    self.nb_neg_broadcast += 1
                if agent.received_broadcasts:
                    last_msg = agent.received_broadcasts[-1]
                    if last_msg.msg_type == "adv" and last_msg.level == agent.level:
                        if compare_adv(last_msg.players_needed, last_msg.missing_stones,
                                       players_needed, missing_resources,
                                       agent.level):
                            agent.neg_broadcast += 1
                            self.nb_neg_broadcast += 1
            else: # action equal to 22 there is no action after
                BROADCAST_TYPE = "inv"
                if agent.received_broadcasts:
                    last_msg = agent.received_broadcasts[-1]
                    if last_msg.msg_type == "adv" and last_msg.level == agent.level:
                        agent.pos_broadcast += 1
                        self.nb_pos_broadcast += 1
                    else:
                        agent.neg_broadcast += 1
                        self.nb_neg_broadcast += 1
                else:
                    agent.neg_broadcast += 1
                    self.nb_neg_broadcast += 1

            msg = Broadcast.create_msg(agent, players_needed, missing_resources, BROADCAST_TYPE)

            for a in (team_agents + other_team):
                if a is agent:
                    continue

                new_msg = copy.copy(msg)
                new_msg.direction = compute_sound_direction(agent, a, self.map_size)

                a.received_broadcasts.append(new_msg)


class Agent:
    def __init__(self, genome, layer_dims, map_size):
        self.genome = genome
        self.layer_dims = layer_dims
        self.map_size = map_size
        self.coordinates = (np.random.randint(map_size[0]), np.random.randint(map_size[1]))
        self.direction = np.random.randint(4)
        self.survival_ticks = 126
        self.level = 1
        self.inventory = {resource: 0 for resource in RESOURCE_KEYS}
        self.dying_tick = -1
        self.received_broadcasts = []
        self.useful_drop = 0
        self.useful_drop_weak = 0
        self.useless_drop = 0
        self.incantation = 0
        self.pos_broadcast = 0
        self.neg_broadcast = 0

        self.current_objective = None
        self.max_age = 100
        self.objective_age = 0
        self.objective_progress = 0
        self.objective_failed = 0
        self.priority = float("inf")

    def act(self, state, valid_mask):
        x   = np.array(state, dtype=np.float32)
        idx = 0
        for i in range(len(self.layer_dims) - 1):
            in_d  = self.layer_dims[i]
            out_d = self.layer_dims[i + 1]
            W = self.genome[idx : idx + in_d * out_d].reshape(out_d, in_d)
            idx += in_d * out_d
            b = self.genome[idx : idx + out_d]
            idx += out_d
            if i < len(self.layer_dims) - 2:
                x = np.maximum(0, W @ x + b)   # ReLU for hidden layers
            else:
                x = W @ x + b                   # no activation on output
        masked = np.where(valid_mask, x, -np.inf)
        return np.argmax(masked)
    
    def get_visible_tiles(self, env, team_agents, other_team):
        result = np.zeros(MAX_VISION_TILES * (len(RESOURCE_KEYS) + 2), dtype=np.float32)

        fx, fy = DIRECTION_VECTORS[self.direction]
        rx, ry = -fy, fx

        ally_positions  = [a.coordinates for a in team_agents  if a is not self and a.dying_tick < 0]
        enemy_positions = [a.coordinates for a in other_team if a.dying_tick < 0]

        idx = 0

        for row in range(self.level + 1):
            for col in range(-row, row + 1):
                nx = (self.coordinates[0] + fx * row + rx * col) % self.map_size[0]
                ny = (self.coordinates[1] + fy * row + ry * col) % self.map_size[1]
                tile = env.map[nx + ny * self.map_size[0]]
                for r in RESOURCE_KEYS:
                    result[idx] = min(tile[r], 10) / 10.0
                    idx += 1
                result[idx]     = min(ally_positions.count((nx, ny)), 5) / 5.0
                result[idx + 1] = min(enemy_positions.count((nx, ny)), 5) / 5.0
                idx += 2
        return result

    def get_state(self, env, team_agents, other_team):
        pos = np.array([
            self.coordinates[0] / self.map_size[0],
            self.coordinates[1] / self.map_size[1],
        ])

        direction = np.zeros(4)
        direction[self.direction] = 1.0

        level = np.array([self.level / 8.0])

        survival = np.array([min(self.survival_ticks, 1260) / 1260.0])

        inventory = np.array([
            min(self.inventory[r], 20) / 20.0 for r in RESOURCE_KEYS
        ])

        best = None
        tile = env.map[self.coordinates[0] + self.coordinates[1] * env.map_size[0]]
        players_needed, missing_stones = compute_adv(self, (team_agents + other_team), tile)
        best_priority = players_needed * 50 + sum(missing_stones) # Maybe even try with 100 later
        inv_reveived = []
        for msg in self.received_broadcasts:
            if msg.msg_type != "adv":
                inv_reveived.append(msg)
            if msg.level != self.level:
                continue

            priority = msg.players_needed * 50 + sum(msg.missing_stones)
            if priority == 0:
                continue
            if priority < best_priority:
                best = msg
                best_priority = priority

        broadcast = np.zeros(20)
        if best is not None:
            self.current_objective = best.direction
            self.priority = best_priority
            self.objective_age = 0
            self.max_age = min(150, 30 + self.priority * 5+ self.level * 20)
            broadcast = best.normalize(broadcast)
            self.received_broadcasts.clear()
        else:
            best_rest = float("inf")
            best_inv = None
            for msg in inv_reveived:
                rest = 0
                for i in range(len(STONE_KEYS)):
                    rest += max(0, missing_stones[i] - msg.missing_stones[i])
                if rest < best_rest:
                    best_rest = rest
                    best_inv = msg
            if best_inv is not None:
                broadcast = best_inv.normalize(broadcast)
            self.received_broadcasts.clear()

        vision = self.get_visible_tiles(env, team_agents, other_team)

        return np.concatenate([pos, direction, level, survival, inventory, broadcast, vision])

    def fitness(self, team, tile):
        needed = EVOLUTION_TABLE[self.level]
        valid_allies = [
            a for a in team
            if a.coordinates == self.coordinates
            and a.level == self.level
        ]
        ally_progress = min(len(valid_allies) / needed[0], 1.0)

        stone_progress = sum(
            min(tile[r] / req, 1.0)
            for r, req in zip(STONE_KEYS, needed[1:])
            if req > 0
        )
        stone_progress /= sum(
            1 for req in needed[1:] if req > 0
        )

        return (
            (self.level**2) * 1000
            + ally_progress * 300
            + stone_progress * 300
            + self.useful_drop * 50
            + min(self.useful_drop_weak, 20) * 20
            - self.useless_drop * 20
            + self.incantation * 300
            + self.pos_broadcast * 20
            - self.neg_broadcast * 30
            + self.objective_progress * 0.5
            - self.objective_failed * 20
            + self.survival_ticks * 0.1
        )

    def compute_prog(self, factor):
        if self.current_objective == 1:
            self.objective_progress += (1 / factor)
            return True
        elif self.current_objective in (2, 8):
            self.objective_progress += (0.5 / factor)
            return True
        return False

    def move(self, move, facing=None):
        dx, dy = DIRECTION_VECTORS[facing if facing is not None else self.direction]
        if move == -1:
            dx, dy = -dx, -dy
        
        self.coordinates = (
            (self.coordinates[0] + dx) % self.map_size[0],
            (self.coordinates[1] + dy) % self.map_size[1],
        )

        if move == 1 and self.current_objective:
            return self.compute_prog(1)
        return False

    def move_to(self, coordinates):
        self.coordinates = coordinates

    def turn(self, direction):
        self.direction = (self.direction + (1 if direction == 1 else -1)) % 4

        if (self.current_objective):
            if direction == 0:
                self.current_objective = LEFT_ROT[self.current_objective]
            else:
                self.current_objective = RIGHT_ROT[self.current_objective]
            
            return self.compute_prog(2)
        return False

    def get_resource(self, resource):
        self.inventory[resource] += 1

    def drop_resource(self, resource, tile):
        self.inventory[resource] -= 1
        if resource != "food":
            idx = STONE_KEYS.index(resource)
            needed_rock = sum(
                EVOLUTION_TABLE[level][1 + idx]
                for level in range(self.level + 1, 8)
            )
            if tile[resource] < EVOLUTION_TABLE[self.level][1 + idx]:
                self.useful_drop += 1
                return True
            elif tile[resource] < needed_rock:
                self.useful_drop_weak += 1
                return True
        self.useless_drop += 1
        return False

    def eat(self):
        self.inventory["food"] -= 1
        self.survival_ticks = 126

    def update(self, tick):
        self.survival_ticks -= 1
        if self.dying_tick >= 0:
            return False
        if self.survival_ticks <= 0:
            self.dying_tick = tick
            return False
        return True


def next_generation(population, scores, elite_frac=0.2, mutation_std=0.02, reinject_frac=0.1):
    best_score = max(scores)
    limit = best_score - 1000

    ranked_pairs = sorted(
        zip(scores, population),
        key=lambda x: x[0],
        reverse=True
    )
    ranked = [g for _, g in ranked_pairs]
    ranked_scores = [s for s, _ in ranked_pairs]

    nb_best = sum(
        1 for score, _ in ranked_pairs
        if score >= limit
    )
    max_elites = int(len(population) * elite_frac)
    n_elite = min(max_elites, nb_best)

    elites = ranked[:n_elite]
    elite_scores = ranked_scores[:n_elite]

    score_range = max(elite_scores) - min(elite_scores)
    collapsed   = (n_elite == max_elites) and (score_range < 1.0)

    n_reinject = int(len(population) * reinject_frac) if collapsed else 0
    n_children = len(population) - n_elite - n_reinject

    children = [
        elites[np.random.randint(len(elites))] +
        np.random.randn(len(elites[0])) * mutation_std
        for _ in range(n_children)
    ]
    randoms = [
        np.random.randn(len(population[0])) * 0.1
        for _ in range(n_reinject)
    ]
    return elites + children + randoms, n_elite #, collapsed (maybe for log later)

def run_episode_two_teams(genomes_a, genomes_b, env, cfg, prev_scores_a, prev_scores_b):
    indices_a = random.sample(
        range(len(genomes_a)),
        min(cfg["TEAM_EPISODE_SIZE"], len(genomes_a))
    )
    indices_b = random.sample(
        range(len(genomes_b)),
        min(cfg["TEAM_EPISODE_SIZE"], len(genomes_b))
    )

    agents_a = [
        Agent(genomes_a[i], cfg["LAYER_DIMS"], env.map_size)
        for i in indices_a
    ]
    agents_b = [
        Agent(genomes_a[i], cfg["LAYER_DIMS"], env.map_size)
        for i in indices_b
    ]

    winner = env.run(agents_a, agents_b)

    highest_lvl_a = max(a.level for a in agents_a)
    highest_lvl_b = max(a.level for a in agents_b)

    full_scores_a = list(prev_scores_a)
    full_scores_b = list(prev_scores_b)
    for idx, agent in zip(indices_a, agents_a):
        full_scores_a[idx] = agent.fitness(agents_a,
                            env.map[agent.coordinates[0] + agent.coordinates[1] * env.map_size[0]])
    for idx, agent in zip(indices_b, agents_b):
        full_scores_b[idx] = agent.fitness(agents_b,
                            env.map[agent.coordinates[0] + agent.coordinates[1] * env.map_size[0]])

    return (full_scores_a, full_scores_b,
            highest_lvl_a, highest_lvl_b,
            winner, len(agents_a), len(agents_b))

def compute_genome_size(layer_dims):
    size = 0
    for i in range(len(layer_dims) - 1):
        size += layer_dims[i] * layer_dims[i+1]
        size += layer_dims[i+1]
    return size

def train(cfg, pop_a, pop_b, log_path):
    consecutive_wins_a = 0
    consecutive_wins_b = 0
    win_threshold = cfg.get("WIN_STREAK_STOP", 50)

    with open(log_path, "w", newline="") as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=[
            "gen", "best_a", "best_b", "mean_a", "mean_b",
            "max_level_a", "max_level_b",
            "team_a_size", "team_b_size", "n_elite_a", "n_elite_b",
            "nb_fork", "nb_incantation",
            "nb_broadcast", "nb_neg_broadcast", "nb_pos_broadcast",
            "useful_drops", "useless_drops",
            "objective_prog", "objective_failed",
            "teammates_reach", "winner"
        ])
        writer.writeheader()

        scores_a = [0.0] * (cfg_params["POP_SIZE"] // cfg_params["N_TEAMS"])
        scores_b = [0.0] * (cfg_params["POP_SIZE"] // cfg_params["N_TEAMS"])

        for gen in range(cfg["N_GENERATIONS"]):
            env = ZappyEnv(cfg["MAP_SIZE"], cfg["MAX_EPISODE_TICKS"], cfg["MAX_TEAM_SIZE"])
            (scores_a, scores_b,
            max_lvl_a, max_lvl_b,
            winner,
            team_a_size, team_b_size) = run_episode_two_teams(pop_a, pop_b, env, cfg, scores_a, scores_b)

            # early stopping
            if winner == 0:
                consecutive_wins_a += 1
                consecutive_wins_b  = 0
            elif winner == 1:
                consecutive_wins_b += 1
                consecutive_wins_a  = 0
            else:
                consecutive_wins_a = 0
                consecutive_wins_b = 0

            if consecutive_wins_a >= win_threshold:
                break
            if consecutive_wins_b >= win_threshold:
                break

            pop_a, n_elite_a = next_generation(pop_a, scores_a, cfg["ELITE_FRAC"], cfg["MUTATION_STD"], cfg["REINJECT_FRAC"])
            pop_b, n_elite_b = next_generation(pop_b, scores_b, cfg["ELITE_FRAC"], cfg["MUTATION_STD"], cfg["REINJECT_FRAC"])

            writer.writerow({
                "gen":              gen,
                "best_a":           max(scores_a),
                "best_b":           max(scores_b),
                "mean_a":           round(np.mean(scores_a), 2),
                "mean_b":           round(np.mean(scores_b), 2),
                "max_level_a":      max_lvl_a,
                "max_level_b":      max_lvl_b,
                "team_a_size":      team_a_size,
                "team_b_size":      team_b_size,
                "n_elite_a":        n_elite_a,
                "n_elite_b":        n_elite_b,
                "nb_fork":          env.nb_fork,
                "nb_incantation":   env.nb_incantation,
                "nb_broadcast":     env.nb_broadcast,
                "nb_neg_broadcast": env.nb_neg_broadcast,
                "nb_pos_broadcast": env.nb_pos_broadcast,
                "useful_drops":     env.useful_drops,
                "useless_drops":    env.useless_drops,
                "objective_prog":   env.objective_prog,
                "objective_failed": env.objective_failed,
                "teammates_reach":  env.teammates_reach,
                "winner":           winner,
            })
            csvfile.flush()

            if gen % cfg.get("CHECKPOINT_INTERVAL", 20) == 0:
                os.makedirs("checkpoints", exist_ok=True)
                np.save(f"checkpoints/gen{gen}_a.npy", pop_a[0])
                np.save(f"checkpoints/gen{gen}_b.npy", pop_b[0])

    return pop_a, pop_b, scores_a, scores_b

def load_stats(log_path):
    int_fields = {"gen", "max_level_a", "max_level_b", "winner"}
    stats = {k: [] for k in [
        "gen", "best_a", "best_b", "mean_a", "mean_b",
        "max_level_a", "max_level_b", "winner"
    ]}
    with open(log_path, newline="") as f:
        for row in csv.DictReader(f):
            for k in stats:
                stats[k].append(int(float(row[k])) if k in int_fields else float(row[k]))
    return stats

def plot_stats(stats, directory_name):
    gens     = range(len(stats["best_a"]))
    window   = 20

    fig, axes = plt.subplots(3, 2, figsize=(14, 15))

    # --- row 0: fitness ---
    ax = axes[0, 0]
    ax.plot(gens, stats["best_a"], label="Team A best",  color="royalblue")
    ax.plot(gens, stats["best_b"], label="Team B best",  color="tomato")
    ax.plot(gens, stats["mean_a"], label="Team A mean",  color="royalblue", linestyle="--", alpha=0.5)
    ax.plot(gens, stats["mean_b"], label="Team B mean",  color="tomato",    linestyle="--", alpha=0.5)
    ax.set_title("Fitness over generations")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Fitness")
    ax.legend()
    ax.grid(True, alpha=0.3)

    # --- row 0: win rate ---
    ax = axes[0, 1]
    winners = np.array(stats["winner"])
    win_a = np.convolve(winners == 0,  np.ones(window) / window, mode="valid")
    win_b = np.convolve(winners == 1,  np.ones(window) / window, mode="valid")
    draw  = np.convolve(winners == -1, np.ones(window) / window, mode="valid")
    x     = range(len(win_a))
    ax.plot(x, win_a, label="Team A", color="royalblue")
    ax.plot(x, win_b, label="Team B", color="tomato")
    ax.plot(x, draw,  label="Draw",   color="grey", linestyle=":")
    ax.set_title(f"Win rate (rolling {window}-gen window)")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Rate")
    ax.set_ylim(0, 1)
    ax.legend()
    ax.grid(True, alpha=0.3)

    # --- row 1: max level reached ---
    ax = axes[1, 0]
    ax.plot(gens, stats["max_level_a"], label="Team A", color="royalblue")
    ax.plot(gens, stats["max_level_b"], label="Team B", color="tomato")
    # rolling max to show progression trend
    roll_a = np.convolve(stats["max_level_a"], np.ones(window) / window, mode="valid")
    roll_b = np.convolve(stats["max_level_b"], np.ones(window) / window, mode="valid")
    ax.plot(range(len(roll_a)), roll_a, color="royalblue", linestyle="--",
            alpha=0.5, label=f"A rolling avg ({window})")
    ax.plot(range(len(roll_b)), roll_b, color="tomato",    linestyle="--",
            alpha=0.5, label=f"B rolling avg ({window})")
    ax.set_title("Max level reached per generation")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Level (1–8)")
    ax.set_yticks(range(1, 9))
    ax.legend()
    ax.grid(True, alpha=0.3)

    # --- row 1: level distribution heatmap ---
    ax = axes[1, 1]
    # count how often each level was the max, per team, across all gens
    level_counts_a = np.zeros(8)
    level_counts_b = np.zeros(8)
    for lvl in stats["max_level_a"]:
        level_counts_a[lvl - 1] += 1
    for lvl in stats["max_level_b"]:
        level_counts_b[lvl - 1] += 1
    x      = np.arange(1, 9)
    width  = 0.35
    ax.bar(x - width/2, level_counts_a, width, label="Team A", color="royalblue", alpha=0.7)
    ax.bar(x + width/2, level_counts_b, width, label="Team B", color="tomato",    alpha=0.7)
    ax.set_title("Distribution of max level reached")
    ax.set_xlabel("Max level")
    ax.set_ylabel("Count across generations")
    ax.set_xticks(x)
    ax.legend()
    ax.grid(True, alpha=0.3, axis="y")

    # --- row 2: draw/timeout rate + cumulative wins ---
    ax = axes[2, 0]
    ax.plot(range(len(draw)), draw, color="grey")
    ax.set_title(f"Draw/timeout rate (rolling {window}-gen window)")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Rate")
    ax.set_ylim(0, 1)
    ax.grid(True, alpha=0.3)

    ax = axes[2, 1]
    cum_a = np.cumsum(winners == 0)
    cum_b = np.cumsum(winners == 1)
    ax.plot(gens, cum_a, label="Team A", color="royalblue")
    ax.plot(gens, cum_b, label="Team B", color="tomato")
    ax.set_title("Cumulative wins")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Total wins")
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(f"{directory_name}/training_stats.png", dpi=150)
    plt.show()

def compute_state_dim():
    n_resources = len(TILE_SPAWN_DENSITY)   # 7
    max_vision_tiles = (7 + 1) ** 2             # 64, level 7 max

    pos       = 2                               # x, y normalized
    direction = 4                               # one-hot
    level     = 1
    survival  = 1
    inventory = n_resources                     # 7
    broadcast = 20                              # 1 bool + 9 direction one-hot (0-8) + missing spec
    vision    = max_vision_tiles * (n_resources + 2)  # 64 * 7 = 448

    total = pos + direction + level + survival + inventory + broadcast + vision
    return total

def parse_yaml():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=str, default="config.yaml")
    args = parser.parse_args()

    with open(args.config, "r") as f:
        return yaml.safe_load(f)

if __name__ == "__main__":
    cfg_params = parse_yaml()

    if cfg_params["POP_SIZE"] % cfg_params["N_TEAMS"] != 0:
        raise ValueError("POP_SIZE must be divisible by N_TEAMS")
    
    directory_name = f"seed_{cfg_params['SEED']}"
    os.makedirs(directory_name, exist_ok=True)
    
    cfg_params["STATE_DIM"]  = compute_state_dim()
    cfg_params["LAYER_DIMS"] = [cfg_params["STATE_DIM"]] + cfg_params["HIDDEN_LAYERS"] + [len(ACTIONS)]

    genome_size = compute_genome_size(cfg_params["LAYER_DIMS"])  # e.g. [state_dim, 128, 64, 22]

    pop_a = [np.random.randn(genome_size) * 0.1 for _ in range(cfg_params["POP_SIZE"] // cfg_params["N_TEAMS"])]
    pop_b = [np.random.randn(genome_size) * 0.1 for _ in range(cfg_params["POP_SIZE"] // cfg_params["N_TEAMS"])]

    log_path = f"{directory_name}/training.csv"
    pop_a, pop_b, scores_a, scores_b = train(cfg_params, pop_a, pop_b, log_path)

    plot_stats(load_stats(log_path), directory_name)

    is_best_score_a = max(scores_a) >= max(scores_b)

    best_genome = pop_a[0] if is_best_score_a else pop_b[0]
    best_team = "a" if is_best_score_a else "b"
    np.save(f"{directory_name}/best_genome.npy", best_genome)

    with open(f"{directory_name}/arch.json", "w") as f:
        json.dump({
            "layer_dims": cfg_params["LAYER_DIMS"],
            "state_dim":  cfg_params["STATE_DIM"],
            "n_actions":  len(ACTIONS),
            "team":       best_team,
        }, f, indent=2)
