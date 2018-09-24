use enum_like::EnumValues;
use enum_set2::EnumSet;

#[derive(Copy, Clone, Debug, EnumLike, PartialEq, Eq)]
enum Suit {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
}

#[derive(Copy, Clone, Debug, EnumLike, PartialEq, Eq)]
enum Rank {
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,
    J,
    Q,
    K,
    A,
}

#[derive(Copy, Clone, Debug, EnumLike, PartialEq, Eq)]
struct Card {
    number: Rank,
    suit: Suit,
}

#[test]
fn card_deck_game() {
    // Generate one deck of cards
    let mut deck: Vec<Card> = Card::values().collect();
    let mut eall = EnumSet::new();
    for &x in &deck {
        eall.insert(x);
    }
    // eall is the set which contains all the cards

    let mut e1 = EnumSet::new();
    let mut e2 = EnumSet::new();
    let mut e3 = EnumSet::new();
    let mut e4 = EnumSet::new();

    // "Randomly" deal one card to each player until the deck is empty
    let seed = 18917;
    while !deck.is_empty() {
        let random_number = seed % deck.len();
        e1.insert(deck.remove(random_number));
        let random_number = seed % deck.len();
        e2.insert(deck.remove(random_number));
        let random_number = seed % deck.len();
        e3.insert(deck.remove(random_number));
        let random_number = seed % deck.len();
        e4.insert(deck.remove(random_number));
    }

    // There are 4*13 cards, so each player must have gotten 13
    assert_eq!(eall.len(), 4*13);
    assert_eq!(e1.len(), 13);
    assert_eq!(e2.len(), 13);
    assert_eq!(e3.len(), 13);
    assert_eq!(e4.len(), 13);

    // Now each player has a unique set of cards, and the union of their cards
    // must be the full set
    let mut e = e1.clone();
    e.union_with(&e2);
    e.union_with(&e3);
    e.union_with(&e4);
    assert_eq!(e, eall);

    // Let's see... who got the 2 of Hearts?
    let card = Card { number: Rank::N2, suit: Suit::Hearts };
    assert_eq!(e1.contains(card), false); // Not you
    assert_eq!(e2.contains(card), false); // Nope
    assert_eq!(e3.contains(card), false); // No
    assert_eq!(e4.contains(card), true); // Yes!

    // Now it turns out we got another deck, but this one is a deck full
    // of anti-cards. If a player has both a card and the equivalent anti-card,
    // they annihilate each other.

    let mut deck: Vec<_> = Card::values().collect();
    let mut a1 = EnumSet::new();
    let mut a2 = EnumSet::new();
    let mut a3 = EnumSet::new();
    let mut a4 = EnumSet::new();
    // change the seed because otherwise we are dealing the same cards
    let seed = seed + 4;
    while !deck.is_empty() {
        let random_number = seed % deck.len();
        a1.insert(deck.remove(random_number));
        let random_number = seed % deck.len();
        a2.insert(deck.remove(random_number));
        let random_number = seed % deck.len();
        a3.insert(deck.remove(random_number));
        let random_number = seed % deck.len();
        a4.insert(deck.remove(random_number));
    }

    // To calculate the final deck after the annihilations efficiently,
    // we use the XOR operation, also known as "symmetric_difference"
    e1.symmetric_difference_with(&a1);
    e2.symmetric_difference_with(&a2);
    e3.symmetric_difference_with(&a3);
    e4.symmetric_difference_with(&a4);

    // Now, the player with the most cards wins!
    let mut ncards = [(e1.len(), 1), (e2.len(), 2), (e3.len(), 3), (e4.len(), 4)];
    ncards.sort();
    // And the winner is...
    assert_eq!(ncards[3], (22, 1)); // Player 1, with 22 cards left!
    assert_eq!(ncards[2], (20, 4));
    assert_eq!(ncards[1], (18, 2));
    assert_eq!(ncards[0], (14, 3));
}
